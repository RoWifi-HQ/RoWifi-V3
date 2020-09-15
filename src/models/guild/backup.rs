use serde::{Serialize, Deserialize};
use super::GuildSettings;
use super::super::{bind::*, blacklist::Blacklist};

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

    #[serde(rename = "Custombinds")]
    #[serde(default)]
    pub custombinds: Vec<BackupCustomBind>,

    #[serde(rename = "Assetbinds")]
    #[serde(default)]
    pub assetbinds: Vec<BackupAssetBind>,

    #[serde(rename = "Blacklists")]
    #[serde(default)]
    pub blacklists: Vec<Blacklist>,
}