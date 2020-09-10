use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::Arc};
use twilight_model::id::RoleId;

use super::Backup;
use crate::cache::CachedRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupGroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>
}

impl Backup for GroupBind {
    type Bind = BackupGroupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind {
        let mut discord_roles = Vec::new();
        for role_id in self.discord_roles.iter() {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.name.clone());
            }
        }

        BackupGroupBind {
            group_id: self.group_id,
            discord_roles
        }
    }
}