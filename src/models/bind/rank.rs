use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RankBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "RbxRankId")]
    pub rank_id: i64,

    #[serde(rename = "RbxGrpRoleId")]
    pub rbx_rank_id: i64,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64
}