mod flags;

use serde::{Deserialize, Serialize};

use crate::{FromRow, id::GuildId};

pub use flags::UserFlags;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoUser {
    pub discord_id: i64,
    pub default_roblox_id: i64,
    pub alts: Vec<i64>,
    pub flags: UserFlags,
    pub patreon_id: Option<i64>,
    pub premium_servers: Vec<GuildId>,
    pub transferred_from: Option<i64>,
    pub transferred_to: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoGuildUser {
    pub guild_id: GuildId,
    pub discord_id: i64,
    pub roblox_id: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct QueueUser {
    pub roblox_id: i64,
    pub discord_id: i64,
    pub verified: bool,
}

impl FromRow for RoUser {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let discord_id = row.try_get("discord_id")?;
        let default_roblox_id = row.try_get("default_roblox_id")?;
        let alts = row.try_get("alts")?;
        let flags = row.try_get("flags")?;
        let patreon_id = row.try_get("patreon_id")?;
        let premium_servers = row.try_get("premium_servers")?;
        let transferred_from = row.try_get("transferred_from").ok();
        let transferred_to = row.try_get("transferred_to").ok();

        Ok(Self {
            discord_id,
            default_roblox_id,
            alts,
            flags,
            patreon_id,
            premium_servers,
            transferred_from,
            transferred_to,
        })
    }
}

impl FromRow for RoGuildUser {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let guild_id = row.try_get("guild_id")?;
        let discord_id = row.try_get("discord_id")?;
        let roblox_id = row.try_get("roblox_id")?;

        Ok(Self {
            guild_id,
            discord_id,
            roblox_id,
        })
    }
}

impl FromRow for QueueUser {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let verified = row.try_get("verified")?;
        let discord_id = row.try_get("discord_id")?;
        let roblox_id = row.try_get("roblox_id")?;

        Ok(Self {
            roblox_id,
            discord_id,
            verified,
        })
    }
}
