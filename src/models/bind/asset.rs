use std::{fmt, str::FromStr, collections::HashMap, sync::Arc};
use serde::{Serialize, Deserialize};
use serde_repr::*;
use twilight_model::id::RoleId;

use super::Backup;
use crate::cache::CachedRole;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Type")]
    pub asset_type: AssetType,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupAssetBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Type")]
    pub asset_type: AssetType,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<String>
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq, Copy, Clone)]
#[repr(i8)]
pub enum AssetType {
    Asset, Badge, Gamepass
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AssetType::Asset => f.write_str("Asset"),
            AssetType::Badge => f.write_str("Badge"),
            AssetType::Gamepass => f.write_str("Gamepass")
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
             _ => Err(())
        }
    }
}

impl Backup for AssetBind {
    type Bind = BackupAssetBind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind {
        let mut discord_roles = Vec::new();
        for role_id in self.discord_roles.iter() {
            if let Some(role) = roles.get(&RoleId(*role_id as u64)) {
                discord_roles.push(role.name.clone());
            }
        }

        BackupAssetBind {
            id: self.id,
            asset_type: self.asset_type,
            discord_roles
        }
    }

    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self {
        let mut discord_roles = Vec::new();
        for role_name in bind.discord_roles.iter() {
            let role = roles.get(role_name).unwrap().0 as i64;
            discord_roles.push(role);
        }

        AssetBind {
            id: bind.id,
            asset_type: bind.asset_type,
            discord_roles
        }
    }
}