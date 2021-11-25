use super::Template;

use crate::FromRow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Groupbind {
    /// The Id of the Roblox Group
    pub group_id: i64,
    /// The discord roles bound to the group
    pub discord_roles: Vec<i64>,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
}

impl FromRow for Groupbind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let group_id = row.try_get("group_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;

        Ok(Self {
            group_id,
            discord_roles,
            priority,
            template,
        })
    }
}