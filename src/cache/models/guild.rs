use std::sync::{atomic::AtomicI64, Arc};
use twilight_model::{
    guild::Permissions,
    id::{ChannelId, GuildId, RoleId, UserId},
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
    /// The channel id to log RoWifi commands to. Currently held by `#rowifi-logs`
    pub log_channel: Option<ChannelId>,
    /// The role id used to bypass the update command and auto detection. Currently held by `RoWifi Bypass`
    pub bypass_role: Option<RoleId>,
    /// The role id to prevent updating the nickname. Currently held by `RoWifi Nickname Bypass`
    pub nickname_bypass: Option<RoleId>,
    /// The role id giving full access to RoWifi commands. Currently held by `RoWifi Admin`
    pub admin_role: Option<RoleId>,
    /// The atomic field holding the current member count.
    /// We don't wanna depend on calling the API everytime we need a member count
    pub member_count: Arc<AtomicI64>,
}
