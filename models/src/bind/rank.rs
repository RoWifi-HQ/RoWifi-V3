use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::RoleId;

use crate::user::RoUser;

use super::{template::Template, Backup, Bind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "RbxRankId")]
    pub rank_id: i64,

    #[serde(rename = "RbxGrpRoleId")]
    pub rbx_rank_id: i64,

    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    #[serde(rename = "Priority")]
    pub priority: i64,

    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
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

    #[serde(rename = "Prefix", skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    #[serde(rename = "Priority")]
    pub priority: i64,

    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

impl Backup for RankBind {
    type BackupBind = BackupRankBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind {
        let mut discord_roles = Vec::new();
        for role_id in &self.discord_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.clone());
            }
        }

        BackupRankBind {
            group_id: self.group_id,
            rank_id: self.rank_id,
            rbx_rank_id: self.rbx_rank_id,
            prefix: self.prefix.clone(),
            priority: self.priority,
            discord_roles,
            template: self.template.clone(),
        }
    }

    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in &bind.discord_roles {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }
        RankBind {
            group_id: bind.group_id,
            rank_id: bind.rank_id,
            rbx_rank_id: bind.rbx_rank_id,
            prefix: bind.prefix.clone(),
            priority: bind.priority,
            discord_roles,
            template: bind.template.clone(),
        }
    }
}

impl Bind for RankBind {
    fn nickname(&self, roblox_username: &str, user: &RoUser, discord_nick: &str) -> String {
        if let Some(template) = &self.template {
            return template.nickname(roblox_username, user, discord_nick);
        } else if let Some(prefix) = &self.prefix {
            if prefix.eq_ignore_ascii_case("N/A") {
                return roblox_username.to_string();
            } else if prefix.eq_ignore_ascii_case("disable") {
                return discord_nick.to_string();
            }
            return format!("{} {}", prefix, roblox_username);
        }
        discord_nick.to_string()
    }

    fn priority(&self) -> i64 {
        self.priority
    }
}
