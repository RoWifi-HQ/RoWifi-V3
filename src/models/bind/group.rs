use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}