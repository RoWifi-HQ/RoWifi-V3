use itertools::Itertools;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

use super::error::RoError;

type Result<T> = std::result::Result<T, RoError>;

#[derive(Clone)]
pub struct Roblox {
    client: Client,
}

impl Roblox {
    pub fn new() -> Self {
        let client = Client::new();
        Self { client }
    }

    pub async fn get_user_roles(&self, roblox_id: i64) -> Result<HashMap<i64, i64>> {
        let url = format!(
            "https://groups.roblox.com/v2/users/{}/groups/roles",
            roblox_id
        );
        let body: Value = self.client.get(&url).send().await?.json::<Value>().await?;

        let resp = body["data"].as_array().unwrap();

        let mut ranks = HashMap::new();
        for rank in resp.iter() {
            ranks.insert(
                rank["group"]["id"].as_i64().unwrap(),
                rank["role"]["rank"].as_i64().unwrap(),
            );
        }
        Ok(ranks)
    }

    pub async fn get_username_from_id(&self, roblox_id: i64) -> Result<String> {
        let url = format!("https://api.roblox.com/users/{}", roblox_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;

        let resp = body["Username"].as_str().unwrap();
        Ok(resp.to_string())
    }

    pub async fn get_id_from_username(&self, username: &str) -> Result<Option<i64>> {
        let url = format!(
            "https://api.roblox.com/users/get-by-username?username={}",
            username
        );
        let body = self.client.get(&url).send().await?.json::<Value>().await?;

        Ok(body["Id"].as_i64())
    }

    pub async fn has_asset(&self, roblox_id: i64, item: i64, asset_type: &str) -> Result<bool> {
        let url = format!(
            "https://inventory.roblox.com/v1/users/{}/items/{}/{}",
            roblox_id, asset_type, item
        );
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        if let Some(data) = body.get("data") {
            let resp = data.as_array().unwrap();
            return Ok(resp.is_empty());
        }
        Ok(false)
    }

    pub async fn check_code(&self, roblox_id: i64, code: &str) -> Result<bool> {
        let url = format!("https://www.roblox.com/users/{}/profile", roblox_id);
        let body = self.client.get(&url).send().await?.text().await?;

        Ok(body.contains(code))
    }

    pub async fn get_group_rank(&self, group_id: i64, rank_id: i64) -> Result<Option<Value>> {
        let url = format!("https://groups.roblox.com/v1/groups/{}/roles", group_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        let ranks_array = match body["roles"].as_array() {
            Some(a) => a,
            None => return Ok(None),
        };
        let rank = match ranks_array
            .iter()
            .find(|r| r["rank"].as_i64().unwrap_or_default() == rank_id)
        {
            Some(r) => r,
            None => return Ok(None),
        };
        Ok(Some(rank.to_owned()))
    }

    pub async fn get_group_ranks(
        &self,
        group_id: i64,
        min_rank: i64,
        max_rank: i64,
    ) -> Result<Vec<Value>> {
        let url = format!("https://groups.roblox.com/v1/groups/{}/roles", group_id);
        let body = self.client.get(&url).send().await?.json::<Value>().await?;
        let ranks_array = match body["roles"].as_array() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };
        let ranks = ranks_array
            .iter()
            .filter_map(|r| {
                let rank = r["rank"].as_i64().unwrap();
                if rank >= min_rank && rank <= max_rank {
                    return Some(r.to_owned());
                }
                None
            })
            .collect_vec();

        Ok(ranks)
    }
}
