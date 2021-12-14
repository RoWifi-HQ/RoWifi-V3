pub mod backup;
mod types;

use serde::{Deserialize, Serialize};

use crate::{FromRow, blacklist::Blacklist};

pub use types::{BlacklistActionType, GuildType};

#[derive(Clone, Debug, Deserialize, Default, Serialize)]
pub struct RoGuild {
    /// The id of the guild
    pub guild_id: i64,

    /// The prefix that is to be used by every command run in the guild
    pub command_prefix: String,

    /// The role meant for unverified users in the guild
    pub verification_roles: Vec<i64>,

    /// The role meant for verified users in the guild
    pub verified_roles: Vec<i64>,

    /// The array containing all the [Blacklist] of the guild
    pub blacklists: Vec<Blacklist>,

    /// The list of channels where commands are disabled
    pub disabled_channels: Vec<i64>,

    /// The list of groups that the guild uses for analytics
    pub registered_groups: Vec<i64>,

    pub auto_detection: bool,

    pub kind: GuildType,

    pub premium_owner: Option<i64>,

    pub blacklist_action: BlacklistActionType,

    pub update_on_join: bool,

    pub admin_roles: Vec<i64>,

    pub trainer_roles: Vec<i64>,

    pub bypass_roles: Vec<i64>,

    pub nickname_bypass_roles: Vec<i64>,

    pub log_channel: Option<i64>,
}

impl RoGuild {
    pub fn new(guild_id: i64) -> Self {
        Self {
            guild_id,
            command_prefix: "!".into(),
            ..Default::default()
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
            log_channel
        })
    }
}
