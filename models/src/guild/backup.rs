use postgres_types::Json;
use serde::{Deserialize, Serialize};

use crate::{blacklist::Blacklist, bind::Bind};

use super::BlacklistActionType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildBackup {
    pub backup_id: i64,
    pub user_id: i64,
    pub name: String,
    pub data: Json<GuildBackupData>
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuildBackupData {
    pub command_prefix: String,
    pub verification_roles: Vec<String>,
    pub verified_roles: Vec<String>,
    pub blacklists: Vec<Blacklist>,
    pub blacklist_action: BlacklistActionType,
    pub update_on_join: bool,
    pub binds: Vec<Bind>
}