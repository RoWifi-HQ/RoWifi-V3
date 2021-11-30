use bitflags::bitflags;
use postgres_types::{FromSql, Type, ToSql, IsNull, to_sql_checked};
use bytes::BytesMut;

use crate::FromRow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoUser {
    pub discord_id: i64,
    pub default_roblox_id: i64,
    pub alts: Vec<i64>,
    pub flags: UserFlags,
    pub patreon_id: Option<i64>,
    pub premium_servers: Vec<i64>,
    pub premium_owner: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoGuildUser {
    pub guild_id: i64,
    pub discord_id: i64,
    pub roblox_id: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueueUser {
    pub roblox_id: i64,
    pub discord_id: i64,
    pub verified: bool
}

bitflags! {
    pub struct UserFlags: i32 {
        const NONE = 0;
        const ALPHA = 1;
        const BETA = 1 << 1;
        const STAFF = 1 << 2;
        const PARTNER = 1 << 3;
    }
}

impl FromRow for RoUser {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let discord_id  = row.try_get("discord_id")?;
        let default_roblox_id = row.try_get("default_roblox_id")?;
        let alts = row.try_get("alts")?;
        let flags = row.try_get("flags")?;
        let patreon_id = row.try_get("patreon_id")?;
        let premium_servers = row.try_get("premium_servers")?;
        let premium_owner = row.try_get("premium_owner")?;

        Ok(Self {
            discord_id,
            default_roblox_id,
            alts,
            flags,
            patreon_id,
            premium_servers,
            premium_owner
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
            roblox_id
        })
    }
}

impl FromRow for QueueUser {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let verified = row.try_get("verified")?;
        let discord_id = row.try_get("discord_id")?;
        let roblox_id = row.try_get("roblox_id")?;

        Ok(Self {
            discord_id,
            roblox_id,
            verified
        })
    }
}

impl<'a> FromSql<'a> for UserFlags {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let bits = i32::from_sql(ty, raw)?;
        Ok(Self::from_bits_truncate(bits))
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}

impl ToSql for UserFlags {
    fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        i32::to_sql(&self.bits, ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}