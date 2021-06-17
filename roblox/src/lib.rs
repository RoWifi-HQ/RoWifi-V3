#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::let_unit_value
)]

pub mod error;

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
    user::PartialUser,
    VecWrapper,
};
use rowifi_redis::{redis::AsyncCommands, RedisPool};
use serde::de::DeserializeOwned;
use std::result::Result as StdResult;

use error::Error;

type Result<T> = StdResult<T, Error>;

#[derive(Clone)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
    redis_pool: RedisPool,
}

impl Client {
    /// Create an instance of the Roblox Client
    #[must_use]
    pub fn new(redis_pool: RedisPool) -> Self {
        let connector = hyper_rustls::HttpsConnector::with_webpki_roots();
        let client = HyperClient::builder().build(connector);
        Self { client, redis_pool }
    }

    /// Common method to make requests with the client
    pub async fn request<T: DeserializeOwned>(
        &self,
        url: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<T> {
        let builder = Request::builder().uri(url).method(method);
        let req = if let Some(bytes) = body {
            let len = bytes.len();
            builder
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .header(CONTENT_LENGTH, len)
                .body(Body::from(bytes))?
        } else {
            builder.body(Body::empty())?
        };

        let res = self.client.request(req).await?;

        let status = res.status();
        if !status.is_success() {
            return Err(Error::APIError(status));
        }

        let mut buf = body::aggregate(res.into_body()).await?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Get the group roles of an user
    pub async fn get_user_roles(&self, user_id: UserId) -> Result<Vec<GroupUserRole>> {
        let url = format!(
            "https://groups.roblox.com/v2/users/{}/groups/roles",
            user_id.0
        );
        let user_roles = self
            .request::<VecWrapper<GroupUserRole>>(&url, Method::GET, None)
            .await?;
        Ok(user_roles.data)
    }

    /// Get a [`PartialUser`] from the username
    pub async fn get_user_from_username(&self, username: &str) -> Result<Option<PartialUser>> {
        let url = "https://users.roblox.com/v1/usernames/users";
        let usernames = vec![username];
        let json = serde_json::json!({ "usernames": usernames });
        let body = serde_json::to_vec(&json)?;
        let mut ids = self
            .request::<VecWrapper<PartialUser>>(url, Method::POST, Some(body))
            .await?
            .data
            .into_iter();
        match ids.next() {
            Some(u) => Ok(Some(u)),
            None => Ok(None),
        }
    }

    /// Get a [`PartialUser`] from the user id
    pub async fn get_user(&self, user_id: UserId) -> Result<PartialUser> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("roblox:u:{}", user_id.0);
        let user: Option<PartialUser> = conn.get(&key).await?;
        if let Some(u) = user {
            Ok(u)
        } else {
            let url = format!("https://users.roblox.com/v1/users/{}", user_id.0);
            let user = self.request::<PartialUser>(&url, Method::GET, None).await?;
            let _: () = conn.set_ex(key, user.clone(), 6 * 3600).await?;
            Ok(user)
        }
    }

    /// Get multiple [`PartialUser`] from their ids
    pub async fn get_users(&self, user_ids: &[UserId]) -> Result<Vec<PartialUser>> {
        let mut conn = self.redis_pool.get().await?;
        let url = "https://users.roblox.com/v1/users";
        let json = serde_json::json!({ "userIds": user_ids });
        let body = serde_json::to_vec(&json)?;
        let users = self
            .request::<VecWrapper<PartialUser>>(url, Method::POST, Some(body))
            .await?;

        let mut pipe = rowifi_redis::redis::pipe();
        let mut pipe = pipe.atomic();
        for user in &users.data {
            let key = format!("roblox:u:{}", user.id.0);
            pipe = pipe.set_ex(key, user.clone(), 6 * 3600);
        }
        let _: () = pipe.query_async(&mut conn).await?;

        Ok(users.data)
    }

    /// Get all ranks of a [`Group`] with its id
    pub async fn get_group_ranks(&self, group_id: GroupId) -> Result<Option<Group>> {
        let url = format!("https://groups.roblox.com/v1/groups/{}/roles", group_id.0);
        let group = self.request::<Group>(&url, Method::GET, None).await;
        match group {
            Ok(g) => Ok(Some(g)),
            Err(Error::APIError(status_code)) => {
                if status_code == StatusCode::BAD_REQUEST {
                    return Ok(None);
                }
                Err(Error::APIError(status_code))
            }
            Err(err) => Err(err),
        }
    }

    /// Get the [`Asset`] from an user's inventory
    pub async fn get_asset(
        &self,
        user_id: UserId,
        asset_id: AssetId,
        asset_type: &str,
    ) -> Result<Option<Asset>> {
        let url = format!(
            "https://inventory.roblox.com/v1/users/{}/items/{}/{}",
            user_id.0, asset_type, asset_id.0
        );
        let mut assets = self
            .request::<VecWrapper<Asset>>(&url, Method::GET, None)
            .await?
            .data
            .into_iter();
        match assets.next() {
            Some(a) => Ok(Some(a)),
            None => Ok(None),
        }
    }
}
