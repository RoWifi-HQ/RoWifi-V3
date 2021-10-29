use super::guild::GuildType;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoUser {
    #[serde(rename = "_id")]
    pub discord_id: i64,

    #[serde(rename = "RobloxId")]
    pub roblox_id: i64,

    #[serde(rename = "Alts", default)]
    pub alts: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoGuildUser {
    #[serde(rename = "GuildId")]
    pub guild_id: i64,

    #[serde(rename = "UserId")]
    pub discord_id: i64,

    #[serde(rename = "RobloxId")]
    pub roblox_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueueUser {
    #[serde(rename = "_id")]
    pub roblox_id: i64,

    #[serde(rename = "DiscordId")]
    pub discord_id: i64,

    #[serde(rename = "Verified")]
    pub verified: bool,
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i16)]
pub enum PremiumType {
    Alpha = 0,
    Beta = 1,
    Staff = 2,
    Council = 3,
    Partner = 4,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PremiumUser {
    #[serde(rename = "_id")]
    pub discord_id: i64,

    #[serde(rename = "Type")]
    pub premium_type: PremiumType,

    #[serde(rename = "PatreonId", skip_serializing_if = "Option::is_none")]
    pub patreon_id: Option<i64>,

    #[serde(rename = "Servers")]
    pub discord_servers: Vec<i64>,

    #[serde(rename = "PremiumOwner", skip_serializing_if = "Option::is_none")]
    pub premium_owner: Option<i64>,

    #[serde(rename = "PatreonOwner", skip_serializing_if = "Option::is_none")]
    pub premium_patreon_owner: Option<i64>,
}

impl From<PremiumType> for GuildType {
    fn from(p_type: PremiumType) -> Self {
        match p_type {
            PremiumType::Alpha | PremiumType::Staff => GuildType::Alpha,
            PremiumType::Beta | PremiumType::Council | PremiumType::Partner => GuildType::Beta,
        }
    }
}

impl From<i32> for PremiumType {
    fn from(p: i32) -> Self {
        match p {
            1 => PremiumType::Beta,
            2 => PremiumType::Staff,
            3 => PremiumType::Council,
            4 => PremiumType::Partner,
            _ => PremiumType::Alpha,
        }
    }
}

impl PremiumType {
    pub fn has_backup(&self) -> bool {
        match self {
            PremiumType::Alpha | PremiumType::Staff => false,
            PremiumType::Beta | PremiumType::Council | PremiumType::Partner => true,
        }
    }
}

impl_redis!(RoUser);
impl_redis!(RoGuildUser);
