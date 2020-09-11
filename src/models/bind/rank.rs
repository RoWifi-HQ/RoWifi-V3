use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::Arc};
use twilight_model::id::RoleId;

use super::Backup;
use crate::cache::CachedRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct RankBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "RbxRankId")]
    pub rank_id: i64,

    #[serde(rename = "RbxGrpRoleId")]
    pub rbx_rank_id: i64,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRankBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "RbxRankId")]
    pub rank_id: i64,

    #[serde(rename = "RbxGrpRoleId")]
    pub rbx_rank_id: i64,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64
}

impl Backup for RankBind {
    type Bind = BackupRankBind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind {
        let mut discord_roles = Vec::new();
        for role_id in self.discord_roles.iter() {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.name.clone());
            }
        }

        BackupRankBind {
            group_id: self.group_id,
            rank_id: self.rank_id,
            rbx_rank_id: self.rbx_rank_id,
            prefix: self.prefix.clone(),
            priority: self.priority,
            discord_roles
        }
    }

    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles.iter() {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }
        RankBind {
            group_id: bind.group_id,
            rank_id: bind.rank_id,
            rbx_rank_id: bind.rbx_rank_id,
            prefix: bind.prefix.clone(),
            priority: bind.priority,
            discord_roles
        }
    }
}