#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::let_unit_value
)]

pub mod error;
mod route;

use deadpool_redis::{redis::AsyncCommands, Pool};
use hyper::{
    body::{self, Buf},
    client::HttpConnector,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client as HyperClient, Method, Request, StatusCode,
};
use hyper_rustls::HttpsConnector;
use rowifi_models::roblox::{
    asset::Asset,
    group::{Group, GroupUserRole},
    id::{AssetId, GroupId, UserId},
    user::{PartialUser, User},
    VecWrapper,
};
use serde::de::DeserializeOwned;
use std::{env, result::Result as StdResult};

use error::{Error, ErrorKind};
use route::Route;

type Result<T> = StdResult<T, Error>;

#[derive(Clone)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
    redis_pool: Pool,
    proxy: Option<String>,
}

impl Client {
    /// Create an instance of the Roblox Client
    #[must_use]
    pub fn new(redis_pool: Pool) -> Self {
        let proxy = env::var("RBX_PROXY").ok();
        let connector = hyper_rustls::HttpsConnector::with_webpki_roots();
        let client = HyperClient::builder().build(connector);
        Self {
            client,
            redis_pool,
            proxy,
        }
    }

    /// Common method to make requests with the client
    pub async fn request<T: DeserializeOwned>(
        &self,
        route: Route<'_>,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<T> {
        let route = route.to_string();
        let builder = match &self.proxy {
            Some(p) => Request::builder()
                .uri(format!("{}?url={}", p, urlencoding::encode(&route)))
                .method(method),
            None => Request::builder().uri(&route).method(method),
        };
        let req = if let Some(bytes) = body {
            let len = bytes.len();
            builder
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .header(CONTENT_LENGTH, len)
                .body(Body::from(bytes))
                .map_err(|source| Error {
                    source: Some(Box::new(source)),
                    kind: ErrorKind::BuildingRequest,
                })?
        } else {
            builder.body(Body::empty()).map_err(|source| Error {
                source: Some(Box::new(source)),
                kind: ErrorKind::BuildingRequest,
            })?
        };

        let res = self.client.request(req).await.map_err(|source| Error {
            source: Some(Box::new(source)),
            kind: ErrorKind::RequestError,
        })?;

        let status = res.status();

        let mut buf = body::aggregate(res.into_body())
            .await
            .map_err(|source| Error {
                source: Some(Box::new(source)),
                kind: ErrorKind::ChunkingResponse,
            })?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        if !status.is_success() {
            return Err(Error {
                source: None,
                kind: ErrorKind::Response {
                    body: bytes,
                    status,
                    route,
                },
            });
        }

        let result = serde_json::from_slice(&bytes).map_err(|source| Error {
            source: Some(Box::new(source)),
            kind: ErrorKind::Json { body: bytes },
        })?;
        Ok(result)
    }

    /// Get the group roles of an user
    pub async fn get_user_roles(&self, user_id: UserId) -> Result<Vec<GroupUserRole>> {
        let route = Route::UserGroupRoles { user_id: user_id.0 };
        let user_roles = self
            .request::<VecWrapper<GroupUserRole>>(route, Method::GET, None)
            .await?;
        Ok(user_roles.data)
    }

    /// Get a [`PartialUser`] from the username
    pub async fn get_user_from_username(&self, username: &str) -> Result<Option<PartialUser>> {
        let route = Route::UsersByUsername;
        let usernames = vec![username];
        let json = serde_json::json!({ "usernames": usernames });
        let body = serde_json::to_vec(&json).map_err(|source| Error {
            source: Some(Box::new(source)),
            kind: ErrorKind::BuildingRequest,
        })?;
        let mut ids = self
            .request::<VecWrapper<PartialUser>>(route, Method::POST, Some(body))
            .await?
            .data
            .into_iter();
        match ids.next() {
            Some(u) => Ok(Some(u)),
            None => Ok(None),
        }
    }

    pub async fn get_user_profile(&self, user_id: UserId) -> Result<User> {
        let route = Route::UserById { user_id: user_id.0 };
        let user = self.request::<User>(route, Method::GET, None).await?;
        Ok(user)
    }

    /// Get a [`PartialUser`] from the user id
    pub async fn get_user(&self, user_id: UserId, bypass_cache: bool) -> Result<PartialUser> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("roblox:u:{}", user_id.0);
        if bypass_cache {
            let user = self
                .get_users(&[user_id])
                .await?
                .into_iter()
                .next()
                .ok_or_else(|| Error {
                    source: None,
                    kind: ErrorKind::Response {
                        body: vec![],
                        status: StatusCode::NOT_FOUND,
                        route: Route::UsersById.to_string(),
                    },
                })?;
            let _: () = conn.set_ex(key, user.clone(), 24 * 3600).await?;
            Ok(user)
        } else {
            let user: Option<PartialUser> = conn.get(&key).await?;
            if let Some(u) = user {
                Ok(u)
            } else {
                let user = self
                    .get_users(&[user_id])
                    .await?
                    .into_iter()
                    .next()
                    .ok_or_else(|| Error {
                        source: None,
                        kind: ErrorKind::Response {
                            body: vec![],
                            status: StatusCode::NOT_FOUND,
                            route: Route::UsersById.to_string(),
                        },
                    })?;
                let _: () = conn.set_ex(key, user.clone(), 24 * 3600).await?;
                Ok(user)
            }
        }
    }

    /// Get multiple [`PartialUser`] from their ids
    pub async fn get_users(&self, user_ids: &[UserId]) -> Result<Vec<PartialUser>> {
        let mut conn = self.redis_pool.get().await?;
        let route = Route::UsersById;
        let json = serde_json::json!({ "userIds": user_ids });
        let body = serde_json::to_vec(&json).map_err(|source| Error {
            source: Some(Box::new(source)),
            kind: ErrorKind::BuildingRequest,
        })?;
        let users = self
            .request::<VecWrapper<PartialUser>>(route, Method::POST, Some(body))
            .await?;

        let mut pipe = deadpool_redis::redis::pipe();
        let mut pipe = pipe.atomic();
        for user in &users.data {
            let key = format!("roblox:u:{}", user.id.0);
            pipe = pipe.set_ex(key, user.clone(), 24 * 3600);
        }
        let _: () = pipe.query_async(&mut conn).await?;

        Ok(users.data)
    }

    /// Get all ranks of a [`Group`] with its id
    pub async fn get_group_ranks(&self, group_id: GroupId) -> Result<Option<Group>> {
        let route = Route::GroupRoles {
            group_id: group_id.0,
        };
        let group = self.request::<Group>(route, Method::GET, None).await;
        match group {
            Ok(g) => Ok(Some(g)),
            Err(err) => {
                if let ErrorKind::Response {
                    body: _,
                    status,
                    route: _,
                } = err.kind()
                {
                    if *status == StatusCode::NOT_FOUND {
                        return Ok(None);
                    }
                }
                Err(err)
            }
        }
    }

    /// Get the [`Asset`] from an user's inventory
    pub async fn get_asset(
        &self,
        user_id: UserId,
        asset_id: AssetId,
        asset_type: &str,
    ) -> Result<Option<Asset>> {
        let route = Route::UserInventoryAsset {
            user_id: user_id.0,
            asset_id: asset_id.0,
            asset_type,
        };
        let mut assets = self
            .request::<VecWrapper<Asset>>(route, Method::GET, None)
            .await?
            .data
            .into_iter();
        match assets.next() {
            Some(a) => Ok(Some(a)),
            None => Ok(None),
        }
    }
}
