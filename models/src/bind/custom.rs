use super::Template;

use crate::{FromRow, rolang::RoCommand};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Custombind {
    /// The global id of the bind
    pub bind_id: i64,
    /// The ID of the Custom Bind
    pub custom_bind_id: i32,
    /// The discord roles bound to the custombind
    pub discord_roles: Vec<i64>,
    /// The code of the bind
    pub code: String,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
    pub command: RoCommand,
}

impl FromRow for Custombind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let bind_id = row.try_get("bind_id")?;
        let custom_bind_id = row.try_get("custom_bind_id")?;
        let discord_roles = row.try_get("discord_roles")?;
        let code: String = row.try_get("code")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;
        let command = RoCommand::new(&code).unwrap();

        Ok(Self {
            bind_id,
            custom_bind_id,
            discord_roles,
            code,
            priority,
            template,
            command
        })
    }
}