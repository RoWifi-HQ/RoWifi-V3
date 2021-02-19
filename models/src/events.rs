use bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventLog {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    #[serde(rename = "GuildId")]
    pub guild_id: i64,

    #[serde(rename = "EventType")]
    pub event_type: i64,

    #[serde(rename = "GuildEventId")]
    pub guild_event_id: i64,

    #[serde(rename = "HostId")]
    pub host_id: i64,

    #[serde(rename = "Timestamp")]
    pub timestamp: DateTime,

    #[serde(rename = "Attendees")]
    #[serde(default)]
    pub attendees: Vec<i64>,

    #[serde(rename = "Notes")]
    pub notes: Option<(String, String)>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventType {
    #[serde(rename = "Id")]
    pub id: i64,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "XP")]
    pub xp: i64,
}
