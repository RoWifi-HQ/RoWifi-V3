use serde::{Serialize, Deserialize};
use serde_repr::*;
use std::{default::Default, fmt};
use super::{bind::*, blacklist::*};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RoGuild {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Prefix")]
    pub command_prefix: Option<String>,

    #[serde(rename = "Settings")]
    pub settings: GuildSettings,

    #[serde(rename = "VerificationRole")]
    pub verification_role: i64,

    #[serde(rename = "VerifiedRole")]
    pub verified_role: i64,

    #[serde(rename = "RankBinds")]
    pub rankbinds: Vec<RankBind>,

    #[serde(rename = "GroupBinds")]
    pub groupbinds: Vec<GroupBind>,

    #[serde(rename = "CustomBinds")]
    #[serde(default)]
    pub custombinds: Vec<CustomBind>,

    #[serde(rename = "AssetBinds")]
    #[serde(default)]
    pub assetbinds: Vec<AssetBind>,

    #[serde(rename = "Blacklists")]
    #[serde(default)]
    pub blacklists: Vec<Blacklist>,

    #[serde(rename = "DisabledChannels")]
    #[serde(default)]
    pub disabled_channels: Vec<i64>
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GuildSettings {
    #[serde(rename = "AutoDetection")]
    pub auto_detection: bool,

    #[serde(rename = "Type")]
    pub guild_type: GuildType,

    #[serde(rename = "BlacklistAction")]
    #[serde(default)]
    pub blacklist_action: BlacklistActionType,

    #[serde(rename = "UpdateOnJoin")]
    #[serde(default)]
    pub update_on_join: bool,

    #[serde(rename = "UpdateOnVerify")]
    #[serde(default)]
    pub update_on_verify: bool
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum GuildType {
    Alpha, Beta, Normal
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum BlacklistActionType {
    None, Kick, Ban
}

impl Default for GuildType {
    fn default() -> Self {
        GuildType::Normal
    }
}

impl Default for BlacklistActionType {
    fn default() -> Self {
        BlacklistActionType::None
    }
}

impl fmt::Display for BlacklistActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlacklistActionType::None => write!(f, "None"),
            BlacklistActionType::Kick=> write!(f, "Kick"),
            BlacklistActionType::Ban => write!(f, "Ban")
        }
    }
}

impl fmt::Display for GuildType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuildType::Alpha => write!(f, "Alpha"),
            GuildType::Beta => write!(f, "Beta"),
            GuildType::Normal => write!(f, "Normal")
        }
    }
}