mod error;

use hyper::{
    body::{self, Buf},
    client::HttpConnector,
    header::{HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client as HyperClient, Method, Request,
};
use hyper_rustls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::borrow::ToOwned;

pub use error::PatreonError;

#[derive(Clone)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
    patreon_key: String,
}

impl Client {
    pub fn new(patreon_key: &str) -> Self {
        let connector = hyper_rustls::HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http1()
            .enable_http2()
            .build();
        let client = HyperClient::builder().build(connector);
        Self {
            client,
            patreon_key: patreon_key.to_string(),
        }
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        url: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<T, PatreonError> {
        let builder = Request::builder().uri(url).method(method).header(
            AUTHORIZATION,
            HeaderValue::from_str(&self.patreon_key).unwrap(),
        );
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
            return Err(PatreonError::APIError(status));
        }

        let mut buf = body::aggregate(res.into_body()).await?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    pub async fn get_patron(
        &self,
        discord_id: u64,
    ) -> Result<(Option<String>, Option<String>), PatreonError> {
        let mut link = Some("https://www.patreon.com/api/oauth2/v2/campaigns/3229705/members?include=currently_entitled_tiers,user&fields%5Buser%5D=social_connections".to_string());
        let mut patreon_id: Option<String> = None;
        while link.is_some() {
            let res = self
                .request::<Value>(&link.unwrap(), Method::GET, None)
                .await?;
            let info = res["included"].as_array().unwrap();
            let users = res["data"].as_array().unwrap();
            for user in info {
                if user["type"].as_str().unwrap().eq("user") {
                    let disc = &user["attributes"]["social_connections"]["discord"]["user_id"];
                    if let Some(Ok(disc)) = disc.as_str().map::<Result<u64, _>, _>(str::parse) {
                        if disc == discord_id {
                            patreon_id = Some(user["id"].as_str().unwrap().to_string());
                            for u in users {
                                if u["relationships"]["user"]["data"]["id"].as_str().unwrap()
                                    == patreon_id.as_ref().unwrap()
                                {
                                    let tiers = u["relationships"]["currently_entitled_tiers"]
                                        ["data"]
                                        .as_array()
                                        .unwrap();
                                    if !tiers.is_empty() {
                                        return Ok((
                                            patreon_id,
                                            tiers[0]["id"].as_str().map(ToOwned::to_owned),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            link = res["links"]["next"].as_str().map(ToOwned::to_owned);
        }
        Ok((patreon_id, None))
    }
}
