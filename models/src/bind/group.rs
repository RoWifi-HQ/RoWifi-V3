use serde::{Deserialize, Serialize};

use super::Template;

use crate::{id::RoleId, serialize_i64_as_string, FromRow};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Groupbind {
    /// The global id of the bind
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub bind_id: i64,
    /// The Id of the Roblox Group
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub group_id: i64,
    /// The discord roles bound to the group
    pub discord_roles: Vec<RoleId>,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GroupbindBackup {
    pub group_id: i64,
    pub discord_roles: Vec<String>,
    pub priority: i32,
    pub template: Template,
}

impl FromRow for Groupbind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let bind_id = row.try_get("bind_id")?;
        let group_id = row.try_get("group_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;

        Ok(Self {
            bind_id,
            group_id,
            discord_roles,
            priority,
            template,
        })
    }
}
