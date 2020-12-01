use super::error::RoError;
use reqwest::{header, Client, ClientBuilder};
use serde_json::Value;
use std::borrow::ToOwned;

#[derive(Clone)]
pub struct Patreon {
    client: Client,
}

impl Patreon {
    pub fn new(patreon_key: &str) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(patreon_key).unwrap(),
        );
        let client = ClientBuilder::new()
            .default_headers(headers)
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn get_patron(
        &self,
        discord_id: u64,
    ) -> Result<(Option<String>, Option<String>), RoError> {
        let mut link = Some("https://www.patreon.com/api/oauth2/v2/campaigns/3229705/members?include=currently_entitled_tiers,user&fields%5Buser%5D=social_connections".to_string());
        while link.is_some() {
            let res: Value = self
                .client
                .get(&link.unwrap())
                .send()
                .await?
                .json::<Value>()
                .await?;
            let info = res["included"].as_array().unwrap();
            let users = res["data"].as_array().unwrap();
            for user in info {
                if user["type"].as_str().unwrap().eq("user") {
                    let disc = &user["attributes"]["social_connections"]["discord"]["user_id"];
                    if let Some(Ok(disc)) = disc.as_str().map::<Result<u64, _>, _>(str::parse) {
                        if disc == discord_id {
                            let patreon_id = user["id"].as_str().unwrap();
                            for u in users {
                                if u["relationships"]["user"]["data"]["id"].as_str().unwrap()
                                    == patreon_id
                                {
                                    let tiers = u["relationships"]["currently_entitled_tiers"]
                                        ["data"]
                                        .as_array()
                                        .unwrap();
                                    if !tiers.is_empty() {
                                        return Ok((
                                            Some(patreon_id.to_string()),
                                            tiers[0]["id"].as_str().map(ToOwned::to_owned),
                                        ));
                                    }
                                    return Ok((Some(patreon_id.to_string()), None));
                                }
                            }
                        }
                    }
                }
            }
            link = res["links"]["next"].as_str().map(ToOwned::to_owned);
        }
        Ok((None, None))
    }
}
