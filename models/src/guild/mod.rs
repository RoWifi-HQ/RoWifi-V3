use std::{str::FromStr, fmt::{Display, Formatter, Result as FmtResult}};
use bytes::BytesMut;
use postgres_types::{ToSql, Type, IsNull, to_sql_checked, FromSql};

use crate::{FromRow, blacklist::Blacklist};

#[derive(Clone, Debug, Default)]
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GuildType {
    Free = 0,
    Alpha = 1,
    Beta = 2,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BlacklistActionType {
    None = 0,
    Kick = 1,
    Ban = 2,
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

impl Default for GuildType {
    fn default() -> Self {
        Self::Free
    }
}

impl Default for BlacklistActionType {
    fn default() -> Self {
        Self::None
    }
}

impl Display for GuildType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            GuildType::Alpha => write!(f, "Alpha"),
            GuildType::Beta => write!(f, "Beta"),
            GuildType::Free => write!(f, "Normal"),
        }
    }
}

impl FromStr for BlacklistActionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(BlacklistActionType::None),
            "kick" => Ok(BlacklistActionType::Kick),
            "ban" => Ok(BlacklistActionType::Ban),
            _ => Err(()),
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

impl ToSql for GuildType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i32::to_sql(&(*self as i32), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for GuildType {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let guild_type = i32::from_sql(ty, raw)?;
        match guild_type {
            0 => Ok(GuildType::Free),
            1 => Ok(GuildType::Alpha),
            2 => Ok(GuildType::Beta),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}

impl ToSql for BlacklistActionType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i32::to_sql(&(*self as i32), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for BlacklistActionType {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let bl_type = i32::from_sql(ty, raw)?;
        match bl_type {
            0 => Ok(BlacklistActionType::None),
            1 => Ok(BlacklistActionType::Kick),
            2 => Ok(BlacklistActionType::Ban),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}