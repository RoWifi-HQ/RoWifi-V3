use twilight_model::{
    guild::Permissions,
    id::{ChannelId, GuildId, UserId, RoleId},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedGuild {
    pub id: GuildId,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub joined_at: Option<String>,
    pub member_count: Option<u64>,
    pub name: String,
    pub owner_id: UserId,
    pub permissions: Option<Permissions>,
    pub preferred_locale: String,
    pub unavailable: bool,

    //Custom Fields
    pub log_channel: Option<ChannelId>,
    pub bypass_role: Option<RoleId>,
    pub nickname_bypass: Option<RoleId>
}