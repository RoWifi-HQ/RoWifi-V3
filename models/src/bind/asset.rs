use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use crate::{id::RoleId, serialize_i64_as_string, FromRow};

use super::template::Template;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Assetbind {
    /// The global id of the bind
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub bind_id: i64,
    /// The ID of the Roblox Asset
    #[serde(serialize_with = "serialize_i64_as_string")]
    pub asset_id: i64,
    /// The type of the Asset. Can be one of Asset, Badge, Gamepass
    pub asset_type: AssetType,
    /// The discord roles bounded to the asset
    pub discord_roles: Vec<RoleId>,
    /// The number that decides whether this bind is chosen for the nickname
    pub priority: i32,
    /// The format of the nickname if this bind is chosen
    pub template: Template,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AssetbindBackup {
    pub asset_id: i64,
    pub asset_type: AssetType,
    pub discord_roles: Vec<String>,
    pub priority: i32,
    pub template: Template,
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Ord, PartialEq, PartialOrd, Serialize_repr)]
#[repr(u8)]
pub enum AssetType {
    Asset = 0,
    Badge = 1,
    Gamepass = 2,
}

impl FromRow for Assetbind {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let bind_id = row.try_get("bind_id")?;
        let asset_id = row.try_get("asset_id")?;
        let asset_type = row.try_get("asset_type")?;
        let discord_roles = row.try_get("discord_roles")?;
        let priority = row.try_get("priority")?;
        let template = row.try_get("template")?;

        Ok(Self {
            bind_id,
            asset_id,
            asset_type,
            discord_roles,
            priority,
            template,
        })
    }
}

impl FromStr for AssetType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asset" => Ok(AssetType::Asset),
            "badge" => Ok(AssetType::Badge),
            "gamepass" => Ok(AssetType::Gamepass),
            _ => Err(()),
        }
    }
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match *self {
            AssetType::Asset => f.write_str("Asset"),
            AssetType::Badge => f.write_str("Badge"),
            AssetType::Gamepass => f.write_str("Gamepass"),
        }
    }
}

impl ToSql for AssetType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn StdError + Sync + Send>> {
        i32::to_sql(&(*self as i32), ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for AssetType {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn StdError + Sync + Send>> {
        let bind_type = i32::from_sql(ty, raw)?;
        match bind_type {
            0 => Ok(AssetType::Asset),
            1 => Ok(AssetType::Badge),
            2 => Ok(AssetType::Gamepass),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}
