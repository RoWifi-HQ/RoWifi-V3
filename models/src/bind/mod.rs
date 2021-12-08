mod rank;
mod template;
mod asset;
mod group;
mod custom;

pub use rank::{Rankbind, RankbindBackup};
pub use template::Template;
pub use group::{Groupbind, GroupbindBackup};
pub use custom::{Custombind, CustombindBackup};
pub use asset::{AssetType, Assetbind, AssetbindBackup};

use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};

use crate::{FromRow, user::RoGuildUser, roblox::user::PartialUser as RobloxUser};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Bind {
    Rank(Rankbind),
    Group(Groupbind),
    Custom(Custombind),
    Asset(Assetbind)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BindBackup {
    Rank(RankbindBackup),
    Group(GroupbindBackup),
    Custom(CustombindBackup),
    Asset(AssetbindBackup)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BindType {
    Rank = 0,
    Group = 1,
    Custom = 2,
    Asset = 3
}

impl Bind {
    pub fn priority(&self) -> i32 {
        match self {
            Bind::Rank(r) => r.priority,
            Bind::Group(g) => g.priority,
            Bind::Custom(c) => c.priority,
            Bind::Asset(a) => a.priority,
        }
    }

    pub fn nickname(&self, roblox_user: &RobloxUser, user: &RoGuildUser, discord_username: &str) -> String {
        match self {
            Bind::Rank(r) => {
                r.template.nickname(roblox_user, user, discord_username)
            },
            Bind::Group(g) => {
                g.template.nickname(roblox_user, user, discord_username)
            },
            Bind::Custom(c) => {
                c.template.nickname(roblox_user, user, discord_username)
            },
            Bind::Asset(a) => {
                a.template.nickname(roblox_user, user, discord_username)
            }
        }
    }

    pub fn discord_roles(&self) -> &[i64] {
        match self {
            Bind::Rank(r) => &r.discord_roles,
            Bind::Group(g) => &g.discord_roles,
            Bind::Custom(c) => &c.discord_roles,
            Bind::Asset(a) => &a.discord_roles
        }
    }

    pub const fn kind(&self) -> BindType {
        match self {
            Self::Rank(_) => BindType::Rank,
            Self::Group(_) => BindType::Group,
            Self::Custom(_) => BindType::Custom,
            Self::Asset(_) => BindType::Asset
        }
    }
}

impl BindBackup {
    pub const fn kind(&self) -> BindType {
        match self {
            Self::Rank(_) => BindType::Rank,
            Self::Group(_) => BindType::Group,
            Self::Custom(_) => BindType::Custom,
            Self::Asset(_) => BindType::Asset
        }
    }

    pub fn discord_roles(&self) -> &[String] {
        match self {
            Self::Rank(r) => &r.discord_roles,
            Self::Group(g) => &g.discord_roles,
            Self::Custom(c) => &c.discord_roles,
            Self::Asset(a) => &a.discord_roles
        }
    }
}

impl FromRow for Bind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let kind = row.try_get("bind_type")?;
        match kind {
            BindType::Rank => Ok(Bind::Rank(Rankbind::from_row(row)?)),
            BindType::Group => Ok(Bind::Group(Groupbind::from_row(row)?)),
            BindType::Custom => Ok(Bind::Custom(Custombind::from_row(row)?)),
            BindType::Asset => Ok(Bind::Asset(Assetbind::from_row(row)?)),
        }
    }
}

impl ToSql for BindType {
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

impl<'a> FromSql<'a> for BindType {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let bind_type = i32::from_sql(ty, raw)?;
        match bind_type {
            0 => Ok(BindType::Rank),
            1 => Ok(BindType::Group),
            2 => Ok(BindType::Custom),
            3 => Ok(BindType::Asset),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}
