#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::must_use_candidate
)]

mod error;
mod models;

use body::Buf;
use hyper::{
    body,
    client::HttpConnector,
    header::{HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client as HyperClient, Method, Request,
};
use hyper_rustls::HttpsConnector;
use serde::de::DeserializeOwned;
use std::result::Result as StdResult;

use error::Error;
use models::{group::GroupUserRole, user::PartialUser, VecWrapper};

type Result<T> = StdResult<T, Error>;

pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new() -> Self {
        let connector = hyper_rustls::HttpsConnector::with_webpki_roots();
        let client = HyperClient::builder().build(connector);

        Self { client }
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
        if !status.is_success() {}

        let mut buf = body::aggregate(res.into_body()).await?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    pub async fn get_user_roles(&self, user_id: u64) -> Result<Vec<GroupUserRole>> {
        let url = format!(
            "https://groups.roblox.com/v2/users/{}/groups/roles",
            user_id
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
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
