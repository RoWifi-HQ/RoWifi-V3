#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

pub mod error;
pub mod models;

use hyper::{
    body::{Buf, self},
    client::HttpConnector,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client as HyperClient, Method, Request, StatusCode,
};
use hyper_rustls::HttpsConnector;
use rowifi_redis::{RedisPool, redis::AsyncCommands};
use serde::de::DeserializeOwned;
use std::result::Result as StdResult;

use error::Error;
use models::{
    asset::Asset,
    group::{Group, GroupUserRole},
    id::{AssetId, GroupId, UserId},
    user::{PartialUser, User},
    VecWrapper,
};

type Result<T> = StdResult<T, Error>;

#[derive(Clone)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
    redis_pool: RedisPool
}

impl Client {
    pub fn new(redis_pool: RedisPool) -> Self {
        let connector = hyper_rustls::HttpsConnector::with_webpki_roots();
        let client = HyperClient::builder().build(connector);
        Self { 
            client,
            redis_pool
        }
    }

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

    pub async fn get_id_from_username(&self, username: &str) -> Result<Option<PartialUser>> {
        let url = "https://users.roblox.com/v1/usernames/users";
        let usernames = vec![username];
        let json = serde_json::json!({"usernames": usernames, "excludeBannedUsers": true});
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

    pub async fn get_user(&self, user_id: UserId) -> Result<User> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("roblox:u:{}", user_id.0);
        let user: Option<User> = conn.get(&key).await?;
        match user {
            Some(u) => Ok(u),
            None => {
                let url = format!("https://users.roblox.com/v1/users/{}", user_id.0);
                let user = self.request::<User>(&url, Method::GET, None).await?;
                let _: () = conn.set_ex(key, user.clone(), 6 * 3600).await?;
                Ok(user)
            }
        }
    }

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
