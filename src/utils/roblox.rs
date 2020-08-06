use reqwest::Client;
use std::collections::HashMap;
use serde_json::Value;

use super::error::RoError;
pub struct Roblox {
    client: Client
}

impl Roblox {
    pub fn new() -> Self {
        let client = Client::new();
        Self {
            client
        }
    }

    pub async fn get_user_roles(&self, roblox_id: i64) -> Result<HashMap<i64, i64>, RoError> {
        let url = format!("https://groups.roblox.com/v2/users/{}/groups/roles", roblox_id);
        let body: Value = self.client.get(&url).send()
            .await?
            .json::<Value>()
            .await?;

        let resp = body["data"].as_array().unwrap();

        let mut ranks = HashMap::new();
        for rank in resp.iter() {
            ranks.insert(rank["group"]["id"].as_i64().unwrap(), rank["role"]["rank"].as_i64().unwrap());
        }
        Ok(ranks)
    }

    pub async fn get_username_from_id(&self, roblox_id: i64) -> Result<String, RoError> {
        let url = format!("https://api.roblox.com/users/{}", roblox_id);
        let body = self.client.get(&url).send()
            .await?
            .json::<Value>()
            .await?;

        let resp = body["Username"].as_str().unwrap();
        Ok(resp.to_string())
    }

    pub async fn get_id_from_username(&self, username: &str) -> Result<Option<String>, RoError> {
        let url = format!("https://api.roblox.com/users/get-by-username?username={}", username);
        let body = self.client.get(&url).send()
            .await?
            .json::<Value>()
            .await?;

        Ok(body["Id"].as_str().and_then(|b| Some(b.to_string())))
    }

    pub async fn has_asset(&self, roblox_id: i64, item: i64, asset_type: &str) -> Result<bool, RoError> {
        let url = format!("https://inventory.roblox.com/v1/users/{}/items/{}/{}", roblox_id, asset_type, item);
        let body = self.client.get(&url).send()
            .await?
            .json::<Value>()
            .await?;
        
        let resp = body["data"].as_array().unwrap();
        Ok(resp.is_empty())
    }
}