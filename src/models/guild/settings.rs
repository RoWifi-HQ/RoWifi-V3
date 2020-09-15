use serde::{Serialize, Deserialize};
use super::{GuildType, BlacklistActionType};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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