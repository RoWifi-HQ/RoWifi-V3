use serde::{Deserialize, Serialize};

use super::id::UserId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: String,
    #[serde(rename = "isBanned")]
    pub is_banned: bool, // TODO: field for created
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialUser {
    pub id: UserId,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

impl_redis!(User);
