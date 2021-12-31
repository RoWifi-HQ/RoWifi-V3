pub mod backup;
mod types;

use serde::{Deserialize, Serialize};

use crate::{
    blacklist::Blacklist,
    id::{ChannelId, GuildId, RoleId, UserId},
    serialize_vec_as_string, FromRow,
};

pub use types::{BlacklistActionType, GuildType};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoGuild {
    /// The id of the guild
    pub guild_id: GuildId,

    /// The prefix that is to be used by every command run in the guild
    pub command_prefix: String,

    /// The role meant for unverified users in the guild
    pub verification_roles: Vec<RoleId>,

    /// The role meant for verified users in the guild
    pub verified_roles: Vec<RoleId>,

    /// The array containing all the [Blacklist] of the guild
    pub blacklists: Vec<Blacklist>,

    /// The list of channels where commands are disabled
    pub disabled_channels: Vec<ChannelId>,

    /// The list of groups that the guild uses for analytics
    #[serde(serialize_with = "serialize_vec_as_string")]
    pub registered_groups: Vec<i64>,

    pub auto_detection: bool,

    pub kind: GuildType,

    pub premium_owner: Option<UserId>,

    pub blacklist_action: BlacklistActionType,

    pub update_on_join: bool,

    pub admin_roles: Vec<RoleId>,

    pub trainer_roles: Vec<RoleId>,

    pub bypass_roles: Vec<RoleId>,

    pub nickname_bypass_roles: Vec<RoleId>,

    pub log_channel: Option<ChannelId>,
}

impl RoGuild {
    #[must_use]
    pub fn new(guild_id: GuildId) -> Self {
        Self {
            guild_id,
            command_prefix: "!".into(),
            verification_roles: Vec::new(),
            verified_roles: Vec::new(),
            blacklists: Vec::new(),
            disabled_channels: Vec::new(),
            registered_groups: Vec::new(),
            auto_detection: false,
            kind: GuildType::Free,
            premium_owner: None,
            blacklist_action: BlacklistActionType::None,
            update_on_join: false,
            admin_roles: Vec::new(),
            trainer_roles: Vec::new(),
            bypass_roles: Vec::new(),
            nickname_bypass_roles: Vec::new(),
            log_channel: None,
        }
    }
}

impl FromRow for RoGuild {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let guild_id = row.try_get("guild_id")?;
        let command_prefix = row.try_get("command_prefix")?;
        let verification_roles = row.try_get("verification_roles")?;
        let verified_roles = row.try_get("verified_roles")?;
        let blacklists = row.try_get("blacklists")?;
        let disabled_channels = row.try_get("disabled_channels")?;
        let registered_groups = row.try_get("registered_groups")?;
        let auto_detection = row.try_get("auto_detection")?;
        let kind = row.try_get("kind")?;
        let premium_owner = row.try_get("premium_owner").ok();
        let blacklist_action = row.try_get("blacklist_action")?;
        let update_on_join = row.try_get("update_on_join")?;
        let admin_roles = row.try_get("admin_roles")?;
        let trainer_roles = row.try_get("trainer_roles")?;
        let bypass_roles = row.try_get("bypass_roles")?;
        let nickname_bypass_roles = row.try_get("nickname_bypass_roles")?;
        let log_channel = row.try_get("log_channel").ok();

        Ok(Self {
            guild_id,
            command_prefix,
            verification_roles,
            verified_roles,
            blacklists,
            disabled_channels,
            registered_groups,
            auto_detection,
            kind,
            premium_owner,
            blacklist_action,
            update_on_join,
            admin_roles,
            trainer_roles,
            bypass_roles,
            nickname_bypass_roles,
            log_channel,
        })
    }
}
