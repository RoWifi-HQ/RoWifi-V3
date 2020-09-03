use serde::{Serialize, Deserialize, Deserializer};
use serde_repr::*;
use std::fmt;
use super::command::RoCommand;

#[derive(Debug, Serialize, Deserialize)]
pub struct RankBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "RbxRankId")]
    pub rank_id: i64,

    #[serde(rename = "RbxGrpRoleId")]
    pub rbx_rank_id: i64,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBind {
    #[serde(rename = "GroupId")]
    pub group_id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}

#[derive(Serialize)]
pub struct CustomBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>,

    #[serde(rename = "Code")]
    pub code: String,

    #[serde(rename = "Prefix")]
    pub prefix: String,

    #[serde(rename = "Priority")]
    pub priority: i64,

    #[serde(skip_serializing)]
    pub command: RoCommand
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetBind {
    #[serde(rename = "_id")]
    pub id: i64,

    #[serde(rename = "Type")]
    pub asset_type: AssetType,

    #[serde(rename = "DiscordRoles")]
    pub discord_roles: Vec<i64>
}

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(i8)]
pub enum AssetType {
    Asset, Badge, Gamepass
}

impl<'de> Deserialize<'de> for CustomBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        pub struct EncodedCustombind {
            #[serde(rename = "_id")]
            pub id: i64,

            #[serde(rename = "DiscordRoles")]
            pub discord_roles: Vec<i64>,

            #[serde(rename = "Code")]
            pub code: String,

            #[serde(rename = "Prefix")]
            pub prefix: String,

            #[serde(rename = "Priority")]
            pub priority: i64
        }

        let input = EncodedCustombind::deserialize(deserializer)?;
        let command = RoCommand::new(&input.code).map_err(serde::de::Error::custom)?;

        Ok(CustomBind {
            id: input.id,
            discord_roles: input.discord_roles,
            code: input.code,
            prefix: input.prefix,
            priority: input.priority,
            command
        })
    }
}

impl fmt::Debug for CustomBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomBind")
            .field("Id", &self.id)
            .field("Discord Roles", &self.discord_roles)
            .field("Code", &self.code)
            .field("Prefix", &self.prefix)
            .field("Priority", &self.priority)
            .finish()
    }
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