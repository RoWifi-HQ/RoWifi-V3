use twilight_model::{
    guild::Permissions,
    id::{ChannelId, GuildId, UserId},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedGuild {
    pub id: GuildId,
    pub description: Option<String>,
    pub discovery_splash: Option<String>,
    pub embed_channel_id: Option<ChannelId>,
    pub embed_enabled: Option<bool>,
    pub icon: Option<String>,
    pub joined_at: Option<String>,
    pub large: bool,
    pub member_count: Option<u64>,
    pub name: String,
    pub owner: Option<bool>,
    pub owner_id: UserId,
    pub permissions: Option<Permissions>,
    pub preferred_locale: String,
    pub splash: Option<String>,
    pub unavailable: bool,
}