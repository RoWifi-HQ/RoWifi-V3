use std::{fmt::{Display, Formatter, Result as FmtResult}, str::FromStr};
use bytes::BytesMut;
use postgres_types::{ToSql, Type, IsNull, to_sql_checked, FromSql};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Ord, PartialEq, PartialOrd, Serialize_repr)]
#[repr(u8)]
pub enum GuildType {
    Free = 0,
    Alpha = 1,
    Beta = 2,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Ord, PartialEq, PartialOrd, Serialize_repr)]
#[repr(u8)]
pub enum BlacklistActionType {
    None = 0,
    Kick = 1,
    Ban = 2,
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

impl Display for BlacklistActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BlacklistActionType::None => f.write_str("None"),
            BlacklistActionType::Kick => f.write_str("Kick"),
            BlacklistActionType::Ban => f.write_str("Ban")
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