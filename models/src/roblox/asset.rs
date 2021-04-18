use serde::{Deserialize, Serialize};

use super::id::AssetId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Asset {
    #[serde(rename = "Id")]
    pub id: AssetId,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub asset_type: String,
}
