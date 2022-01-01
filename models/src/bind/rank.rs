use serde::{Deserialize, Serialize};

use crate::{
    id::{BindId, RoleId},
    serialize_i64_as_string, FromRow,
};

use super::template::Template;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Rankbind {
    /// The global id of the bind
    pub bind_id: BindId,
    /// The Id of the Group
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub group_id: i64,
    /// The discord roles bound to the rank
    pub discord_roles: Vec<RoleId>,
    /// The Id of the rank in the group (0-255)
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub group_rank_id: i64,
    /// The global id of the rank
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub roblox_rank_id: i64,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RankbindBackup {
    pub group_id: i64,
    pub discord_roles: Vec<String>,
    pub group_rank_id: i64,
    pub roblox_rank_id: i64,
    pub priority: i32,
    pub template: Template,
}

impl FromRow for Rankbind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let bind_id = row.try_get("bind_id")?;
        let group_id = row.try_get("group_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let group_rank_id = row.try_get("group_rank_id")?;
        let roblox_rank_id = row.try_get("roblox_rank_id")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;

        Ok(Self {
            bind_id,
            group_id,
            discord_roles,
            group_rank_id,
            roblox_rank_id,
            priority,
            template,
        })
    }
}
