use serde::{Deserialize, Serialize};

use super::id::{GroupId, RoleId};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GroupUserRole {
    pub group: PartialGroup,
    pub role: PartialRole,
}

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
}
