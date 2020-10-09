use std::sync::{Arc, atomic::AtomicI64};
use twilight_model::{
    guild::Permissions,
    id::{ChannelId, GuildId, UserId, RoleId},
};

#[derive(Debug, Clone)]
pub struct CachedGuild {
    pub id: GuildId,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub joined_at: Option<String>,
    pub name: String,
    pub owner_id: UserId,
    pub permissions: Option<Permissions>,
    pub preferred_locale: String,
    pub unavailable: bool,

    //Custom Fields
    pub log_channel: Option<ChannelId>,
    pub bypass_role: Option<RoleId>,
    pub nickname_bypass: Option<RoleId>,
    pub admin_role: Option<RoleId>,
    pub member_count: Arc<AtomicI64>
}