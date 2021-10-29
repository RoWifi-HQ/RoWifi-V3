use serde::{Deserialize, Serialize};

use super::id::AssetId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Asset {
    #[serde(rename = "id")]
    pub id: AssetId,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
}
