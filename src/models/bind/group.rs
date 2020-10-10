use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use twilight_model::id::RoleId;

use super::Backup;
use crate::cache::CachedRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupGroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,
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
            discord_roles,
        }
    }

    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles.iter() {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }

        GroupBind {
            group_id: bind.group_id,
            discord_roles,
        }
    }
}
