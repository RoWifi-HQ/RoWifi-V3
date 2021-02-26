use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::RoleId;

use crate::user::RoUser;

use super::{Backup, Bind, template::Template};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "Priority", default)]
    pub priority: i64,

    #[serde(rename = "Template")]
    pub template: Option<Template>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupGroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "Priority", default)]
    pub priority: i64,

    #[serde(rename = "Template")]
    pub template: Option<Template>
}

impl Backup for GroupBind {
    type BackupBind = BackupGroupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind {
        let mut discord_roles = Vec::new();
        for role_id in &self.discord_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.clone());
            }
        }

        BackupGroupBind {
            group_id: self.group_id,
            discord_roles,
            priority: self.priority,
            template: self.template.clone()
        }
    }

    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in &bind.discord_roles {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }

        GroupBind {
            group_id: bind.group_id,
            discord_roles,
            priority: bind.priority,
            template: bind.template.clone()
        }
    }
}

impl Bind for GroupBind {
    fn nickname(&self, roblox_username: &str, user: &RoUser, discord_nick: &str) -> String {
        if let Some(template) = &self.template {
            return template.nickname(roblox_username, user, discord_nick);
        }
        discord_nick.to_string()
    }

    fn priority(&self) -> i64 {
        self.priority
    }
}
