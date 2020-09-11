use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, sync::Arc};
use twilight_model::id::{RoleId, GuildId};

use super::Backup;
use crate::cache::CachedRole;
use crate::framework::context::Context;

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

#[async_trait]
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

    async fn from_backup(ctx: &Context, guild_id: GuildId, bind: Self::Bind, roles: &Vec<Arc<CachedRole>>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles {
            let role = match roles.iter().find(|r| r.name.eq_ignore_ascii_case(&role_name)) {
                Some(r) => r.id.0 as i64,
                None => {
                    let role = ctx.http.create_role(guild_id).name(role_name).await.expect("Error creating a role");
                    role.id.0 as i64
                }
            };
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