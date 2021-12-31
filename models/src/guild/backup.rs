use postgres_types::Json;
use serde::{Deserialize, Serialize};

use crate::{bind::BindBackup, blacklist::Blacklist, FromRow, id::UserId};

use super::BlacklistActionType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildBackup {
    pub backup_id: i64,
    pub discord_id: UserId,
    pub name: String,
    pub data: Json<GuildBackupData>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuildBackupData {
    pub command_prefix: String,
    pub verification_roles: Vec<String>,
    pub verified_roles: Vec<String>,
    pub blacklists: Vec<Blacklist>,
    pub blacklist_action: BlacklistActionType,
    pub update_on_join: bool,
    pub binds: Vec<BindBackup>,
}

impl FromRow for GuildBackup {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let backup_id = row.try_get("backup_id")?;
        let discord_id = row.try_get("discord_id")?;
        let name = row.try_get("name")?;
        let data = row.try_get("data")?;

        Ok(Self {
            backup_id,
            discord_id,
            name,
            data,
        })
    }
}
