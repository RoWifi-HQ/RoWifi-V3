use std::{fmt, str::FromStr};
use serde::{Serialize, Deserialize};
use serde_repr::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Type")]
    pub asset_type: AssetType,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq)]
#[repr(i8)]
pub enum AssetType {
    Asset, Badge, Gamepass
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            AssetType::Asset => f.write_str("Asset"),
            AssetType::Badge => f.write_str("Badge"),
            AssetType::Gamepass => f.write_str("Gamepass")
        }
    }
}

impl FromStr for AssetType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "asset" => Ok(AssetType::Asset),
            "badge" => Ok(AssetType::Badge),
            "gamepass" => Ok(AssetType::Gamepass),
             _ => Err(())
        }
    }
}