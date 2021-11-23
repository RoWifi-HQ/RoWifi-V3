use crate::FromRow;

use super::template::Template;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Rankbind {
    /// The Id of the Group
    pub group_id: i64,
    /// The discord roles bound to the rank
    pub discord_roles: Vec<i64>,
    /// The Id of the rank in the group (0-255)
    pub group_rank_id: i64,
    /// The global id of the rank
    pub roblox_rank_id: i64,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
}

impl FromRow for Rankbind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let group_id = row.try_get("group_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let group_rank_id = row.try_get("group_rank_id")?;
        let roblox_rank_id = row.try_get("roblox_rank_id")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;

        Ok(Self {
            group_id,
            discord_roles,
            group_rank_id,
            roblox_rank_id,
            priority,
            template,
        })
    }
}
