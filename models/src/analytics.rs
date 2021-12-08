use postgres_types::{ToSql, FromSql};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::FromRow;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Group {
    pub group_id: i64,
    pub roles: Vec<Role>,
    pub member_count: i64,
    pub timestamp: DateTime<Utc>
}

#[derive(Clone, Debug, Deserialize, Eq, FromSql, PartialEq, Serialize, ToSql)]
#[postgres(name = "analytics_role")]
pub struct Role {
    pub id: i64,
    pub rank: i64,
    pub member_count: i64
}

impl FromRow for Group {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, tokio_postgres::Error> {
        let group_id = row.try_get("group_id")?;
        let roles = row.try_get("roles")?;
        let member_count = row.try_get("member_count")?;
        let timestamp = row.try_get("timestamp")?;

        Ok(Self {
            group_id,
            roles,
            member_count,
            timestamp
        })
    }
}