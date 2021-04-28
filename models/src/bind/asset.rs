use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};
use twilight_model::id::RoleId;

use crate::roblox::user::PartialUser as RobloxUser;
use crate::user::RoGuildUser;

use super::{template::Template, Backup, Bind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetBind {
    /// The ID of the Roblox Asset
    #[serde(rename = "_id")]
    pub id: i64,
    /// The type of the Asset. Can be one of Asset, Badge, Gamepass
    #[serde(rename = "Type")]
    pub asset_type: AssetType,
    /// The discord roles bounded to the asset
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
pub struct BackupAssetBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Type")]
    pub asset_type: AssetType,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>,

    #[serde(rename = "Priority", default)]
    pub priority: i64,

    #[serde(rename = "Template", skip_serializing_if = "Option::is_none")]
    pub template: Option<Template>,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq, Copy, Clone)]
#[repr(i8)]
pub enum AssetType {
    Asset,
    Badge,
    Gamepass,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            AssetType::Asset => f.write_str("Asset"),
            AssetType::Badge => f.write_str("Badge"),
            AssetType::Gamepass => f.write_str("Gamepass"),
        }
    }
}

impl FromStr for AssetType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asset" => Ok(AssetType::Asset),
            "badge" => Ok(AssetType::Badge),
            "gamepass" => Ok(AssetType::Gamepass),
            _ => Err(()),
        }
    }
}

impl Backup for AssetBind {
    type BackupBind = BackupAssetBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind {
        let mut discord_roles = Vec::new();
        for role_id in &self.discord_roles {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.clone());
            }
        }

        BackupAssetBind {
            id: self.id,
            asset_type: self.asset_type,
            discord_roles,
            priority: self.priority,
            template: self.template.clone(),
        }
    }

    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in &bind.discord_roles {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }

        AssetBind {
            id: bind.id,
            asset_type: bind.asset_type,
            discord_roles,
            priority: bind.priority,
            template: bind.template.clone(),
        }
    }
}

impl Bind for AssetBind {
    fn nickname(&self, roblox_user: &RobloxUser, user: &RoGuildUser, discord_nick: &str) -> String {
        if let Some(template) = &self.template {
            return template.nickname(roblox_user, user, discord_nick);
        }
        roblox_user.name.clone()
    }

    fn priority(&self) -> i64 {
        self.priority
    }
}
