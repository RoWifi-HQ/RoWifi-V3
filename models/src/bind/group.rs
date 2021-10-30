use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_model::id::RoleId;

use crate::roblox::user::PartialUser as RobloxUser;
use crate::user::RoGuildUser;

use super::{template::Template, Backup, Bind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupBind {
    /// The Id of the Roblox Group
    #[serde(rename = "GroupId")]
    pub group_id: i64,
    /// The discord roles bound to the group
    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,
    /// The number that decides whether this bind is chosen for the nickname
    #[serde(rename = "Priority", default)]
    pub priority: i64,
    /// The format of the nickname if this bind is chosen
    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupGroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "Priority", default)]
    pub priority: i64,

    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

impl Backup for GroupBind {
    type BackupBind = BackupGroupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind {
        let mut discord_roles = Vec::new();
        for role_id in &self.discord_roles {
            if let Some(role) = roles.get(&RoleId::new(*role_id as u64).unwrap()) {
                discord_roles.push(role.clone());
            }
        }

        BackupGroupBind {
            group_id: self.group_id,
            discord_roles,
            priority: self.priority,
            template: self.template.clone(),
        }
    }

    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in &bind.discord_roles {
            let role = roles.get(role_name).unwrap().0.get() as i64;
            discord_roles.push(role);
        }

        GroupBind {
            group_id: bind.group_id,
            discord_roles,
            priority: bind.priority,
            template: bind.template.clone(),
        }
    }
}

impl Bind for GroupBind {
    fn nickname(
        &self,
        roblox_user: &RobloxUser,
        user: &RoGuildUser,
        discord_username: &str,
        _discord_nick: &Option<String>,
    ) -> String {
        if let Some(template) = &self.template {
            return template.nickname(roblox_user, user, discord_username);
        }
        roblox_user.name.clone()
    }

    fn priority(&self) -> i64 {
        self.priority
    }
}
