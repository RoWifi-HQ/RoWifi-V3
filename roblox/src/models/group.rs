use serde::{Deserialize, Serialize};

use super::id::{GroupId, RoleId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialGroup {
    pub id: GroupId,
    pub name: String,
    #[serde(rename = "memberCount")]
    pub member_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialRole {
    pub id: RoleId,
    pub name: String,
    pub rank: u8,
    #[serde(rename = "memberCount")]
    pub member_count: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Group {
    #[serde(rename = "groupId")]
    pub id: GroupId,
    #[serde(default)]
    pub roles: Vec<PartialRole>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupUserRole {
    pub group: PartialGroup,
    pub role: PartialRole,
}
