use serde::{Serialize, Deserialize};
use serde_repr::*;
use std::default::Default;
use super::{bind::*, blacklist::*};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GuildSettings {
    #[serde(rename = "AutoDetection")]
    pub auto_detection: bool,

    #[serde(rename = "Type")]
    pub guild_type: GuildType,

    #[serde(rename = "BlacklistAction")]
    #[serde(default)]
    pub blacklist_action: BlacklistActionType,

    #[serde(rename = "UpdateOnJoin")]
    pub update_on_join: Option<bool>,

    #[serde(rename = "UpdateOnVerify")]
    pub update_on_verify: Option<bool>
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

impl Default for BlacklistActionType {
    fn default() -> Self {
        BlacklistActionType::None
    }
}