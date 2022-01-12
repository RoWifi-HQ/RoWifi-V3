use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    id::{EventId, EventTypeId, GuildId},
    FromRow,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventLog {
    pub event_id: EventId,
    pub guild_id: GuildId,
    pub event_type: i32,
    pub guild_event_id: i64,
    pub host_id: i64,
    pub timestamp: DateTime<Utc>,
    pub attendees: Vec<i64>,
    pub notes: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EventType {
    pub event_type_id: EventTypeId,
    pub event_type_guild_id: i32,
    pub guild_id: GuildId,
    pub name: String,
    pub disabled: bool,
}

impl FromRow for EventLog {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let event_id = row.try_get("event_id")?;
        let guild_id = row.try_get("guild_id")?;
        let event_type = row.try_get("event_type")?;
        let guild_event_id = row.try_get("guild_event_id")?;
        let host_id = row.try_get("host_id")?;
        let timestamp = row.try_get("timestamp")?;
        let attendees = row.try_get("attendees")?;
        let notes = row.try_get("notes").ok();

        Ok(Self {
            event_id,
            guild_id,
            event_type,
            guild_event_id,
            host_id,
            timestamp,
            attendees,
            notes,
        })
    }
}

impl FromRow for EventType {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let event_type_id = row.try_get("event_type_id")?;
        let event_type_guild_id = row.try_get("event_type_guild_id")?;
        let guild_id = row.try_get("guild_id")?;
        let name = row.try_get("name")?;
        let disabled = row.try_get("disabled")?;

        Ok(Self {
            event_type_id,
            event_type_guild_id,
            guild_id,
            name,
            disabled,
        })
    }
}
