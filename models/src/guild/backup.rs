use super::GuildSettings;
use crate::{
    bind::{BackupAssetBind, BackupCustomBind, BackupGroupBind, BackupRankBind},
    blacklist::Blacklist,
    events::EventType,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BackupGuild {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,

    #[serde(rename = "UserId")]
    pub user_id: i64,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Prefix")]
    pub command_prefix: Option<String>,

    #[serde(rename = "Settings")]
    pub settings: GuildSettings,

    #[serde(rename = "VerificationRole")]
    pub verification_role: Option<String>,

    #[serde(rename = "VerifiedRole")]
    pub verified_role: Option<String>,

    #[serde(rename = "Rankbinds")]
    pub rankbinds: Vec<BackupRankBind>,

    #[serde(rename = "Groupbinds")]
    pub groupbinds: Vec<BackupGroupBind>,

    #[serde(rename = "Custombinds", default)]
    pub custombinds: Vec<BackupCustomBind>,

    #[serde(rename = "Assetbinds", default)]
    pub assetbinds: Vec<BackupAssetBind>,

    #[serde(rename = "Blacklists", default)]
    pub blacklists: Vec<Blacklist>,

    #[serde(rename = "RegisteredGroups", default)]
    pub registered_groups: Vec<i64>,

    #[serde(rename = "EventTypes", default)]
    pub event_types: Vec<EventType>,
}
