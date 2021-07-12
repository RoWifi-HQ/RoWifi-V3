use super::{BackupGuildSettings, BlacklistActionType, GuildType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use twilight_http::Client as DiscordClient;
use twilight_model::id::{ChannelId, GuildId, RoleId};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GuildSettings {
    #[serde(rename = "AutoDetection")]
    pub auto_detection: bool,

    #[serde(rename = "Type")]
    pub guild_type: GuildType,

    #[serde(rename = "BlacklistAction", default)]
    pub blacklist_action: BlacklistActionType,

    #[serde(rename = "UpdateOnJoin", default)]
    pub update_on_join: bool,

    #[serde(rename = "AdminRoles", default)]
    pub admin_roles: Vec<i64>,

    #[serde(rename = "TrainerRoles", default)]
    pub trainer_roles: Vec<i64>,

    #[serde(rename = "BypassRoles", default)]
    pub bypass_roles: Vec<i64>,

    #[serde(rename = "NicknameBypassRoles", default)]
    pub nickname_bypass_roles: Vec<i64>,

    #[serde(rename = "LogChannel", default)]
    pub log_channel: Option<i64>,
}

impl GuildSettings {
    pub fn to_backup(
        &self,
        roles: &HashMap<RoleId, String>,
        channels: &HashMap<ChannelId, String>,
    ) -> BackupGuildSettings {
        let mut admin_roles = Vec::new();
        for role_id in &self.admin_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                admin_roles.push(role.clone());
            }
        }

        let mut trainer_roles = Vec::new();
        for role_id in &self.admin_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                trainer_roles.push(role.clone());
            }
        }

        let mut bypass_roles = Vec::new();
        for role_id in &self.admin_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                bypass_roles.push(role.clone());
            }
        }

        let mut nickname_bypass_roles = Vec::new();
        for role_id in &self.admin_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                nickname_bypass_roles.push(role.clone());
            }
        }

        let log_channel = match self.log_channel {
            Some(log_channel) => channels.get(&ChannelId(log_channel as u64)).cloned(),
            None => None,
        };

        BackupGuildSettings {
            auto_detection: self.auto_detection,
            guild_type: self.guild_type,
            blacklist_action: self.blacklist_action,
            update_on_join: self.update_on_join,
            admin_roles,
            trainer_roles,
            bypass_roles,
            nickname_bypass_roles,
            log_channel,
        }
    }

    pub async fn from_backup(
        http: DiscordClient,
        backup_settings: BackupGuildSettings,
        guild_id: GuildId,
        names_to_ids: &HashMap<String, RoleId>,
        existing_roles: &[(RoleId, String)],
        existing_channels: &HashMap<String, ChannelId>,
    ) -> Self {
        let mut admin_roles = Vec::new();
        for role in backup_settings.admin_roles {
            let role_id = if let Some(r) = names_to_ids.get(&role) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&role)) {
                (r.0).0 as i64
            } else {
                let role = http.create_role(guild_id).name(role).await.unwrap();
                role.id.0 as i64
            };
            admin_roles.push(role_id);
        }

        let mut trainer_roles = Vec::new();
        for role in backup_settings.trainer_roles {
            let role_id = if let Some(r) = names_to_ids.get(&role) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&role)) {
                (r.0).0 as i64
            } else {
                let role = http.create_role(guild_id).name(role).await.unwrap();
                role.id.0 as i64
            };
            trainer_roles.push(role_id);
        }

        let mut bypass_roles = Vec::new();
        for role in backup_settings.bypass_roles {
            let role_id = if let Some(r) = names_to_ids.get(&role) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&role)) {
                (r.0).0 as i64
            } else {
                let role = http.create_role(guild_id).name(role).await.unwrap();
                role.id.0 as i64
            };
            bypass_roles.push(role_id);
        }

        let mut nickname_bypass_roles = Vec::new();
        for role in backup_settings.nickname_bypass_roles {
            let role_id = if let Some(r) = names_to_ids.get(&role) {
                r.0 as i64
            } else if let Some(r) = existing_roles.iter().find(|e| e.1.eq(&role)) {
                (r.0).0 as i64
            } else {
                let role = http.create_role(guild_id).name(role).await.unwrap();
                role.id.0 as i64
            };
            nickname_bypass_roles.push(role_id);
        }

        let mut log_channel = None;
        if let Some(backup_channel) = &backup_settings.log_channel {
            if let Some(channel) = existing_channels.get(backup_channel) {
                log_channel = Some(channel.0 as i64);
            }
        }

        Self {
            auto_detection: backup_settings.auto_detection,
            guild_type: backup_settings.guild_type,
            blacklist_action: backup_settings.blacklist_action,
            update_on_join: backup_settings.update_on_join,
            admin_roles,
            trainer_roles,
            bypass_roles,
            nickname_bypass_roles,
            log_channel,
        }
    }
}
