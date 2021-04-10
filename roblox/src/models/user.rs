use serde::{Deserialize, Serialize};

use super::id::UserId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialUser {
    pub id: UserId,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}
