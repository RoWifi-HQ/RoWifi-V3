use bytes::BytesMut;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::rolang::{RoCommand, RoCommandUser};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blacklist {
    pub blacklist_id: i64,
    pub reason: String,
    pub data: BlacklistData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlacklistData {
    User(i64),
    Group(i64),
    Custom(RoCommand),
}

#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, Ord, PartialEq, PartialOrd, Serialize_repr)]
#[repr(u8)]
#[non_exhaustive]
pub enum BlacklistType {
    User = 0,
    Group = 1,
    Custom = 2,
}

#[derive(Debug, Deserialize, FromSql, Serialize, ToSql)]
#[postgres(name = "blacklist")]
struct BlacklistIntermediary {
    pub blacklist_id: i64,
    pub reason: String,
    pub kind: BlacklistType,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub code: Option<String>,
}

impl Blacklist {
    #[must_use]
    pub const fn kind(&self) -> BlacklistType {
        match self.data {
            BlacklistData::User(_) => BlacklistType::User,
            BlacklistData::Group(_) => BlacklistType::Group,
            BlacklistData::Custom(_) => BlacklistType::Custom,
        }
    }

    pub fn evaluate(&self, user: &RoCommandUser) -> Result<bool, String> {
        match &self.data {
            BlacklistData::User(u) => Ok(user.user.roblox_id == *u),
            BlacklistData::Group(id) => Ok(user.ranks.contains_key(id)),
            BlacklistData::Custom(cmd) => Ok(cmd.evaluate(user)?),
        }
    }
}

impl ToSql for Blacklist {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let (user_id, group_id, code) = match &self.data {
            BlacklistData::User(u) => (Some(*u), None, None),
            BlacklistData::Group(g) => (None, Some(*g), None),
            BlacklistData::Custom(c) => (None, None, Some(c.code.clone())),
        };
        let intermediary = BlacklistIntermediary {
            blacklist_id: self.blacklist_id,
            reason: self.reason.clone(),
            kind: self.kind(),
            user_id,
            group_id,
            code,
        };
        BlacklistIntermediary::to_sql(&intermediary, ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <BlacklistIntermediary as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Blacklist {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let blacklist_intermediary = BlacklistIntermediary::from_sql(ty, raw)?;
        let data = match blacklist_intermediary.kind {
            BlacklistType::User => BlacklistData::User(blacklist_intermediary.user_id.unwrap()),
            BlacklistType::Group => BlacklistData::Group(blacklist_intermediary.group_id.unwrap()),
            BlacklistType::Custom => BlacklistData::Custom(
                RoCommand::new(&blacklist_intermediary.code.unwrap()).unwrap(),
            ),
        };
        Ok(Blacklist {
            blacklist_id: blacklist_intermediary.blacklist_id,
            reason: blacklist_intermediary.reason,
            data,
        })
    }

    fn accepts(ty: &Type) -> bool {
        <BlacklistIntermediary as FromSql>::accepts(ty)
    }
}

impl Display for BlacklistType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            BlacklistType::User => f.write_str("User"),
            BlacklistType::Group => f.write_str("Group"),
            BlacklistType::Custom => f.write_str("Custom"),
        }
    }
}

impl ToSql for BlacklistType {
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

impl<'a> FromSql<'a> for BlacklistType {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let blacklist_type = i32::from_sql(ty, raw)?;
        match blacklist_type {
            0 => Ok(BlacklistType::User),
            1 => Ok(BlacklistType::Group),
            2 => Ok(BlacklistType::Custom),
            _ => unreachable!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i32 as FromSql>::accepts(ty)
    }
}

impl<'de> Deserialize<'de> for Blacklist {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let intermediary = BlacklistIntermediary::deserialize(deserializer)?;
        let data = match intermediary.kind {
            BlacklistType::User => BlacklistData::User(intermediary.user_id.unwrap()),
            BlacklistType::Group => BlacklistData::Group(intermediary.group_id.unwrap()),
            BlacklistType::Custom => {
                BlacklistData::Custom(RoCommand::new(&intermediary.code.unwrap()).unwrap())
            }
        };
        Ok(Blacklist {
            blacklist_id: intermediary.blacklist_id,
            reason: intermediary.reason,
            data,
        })
    }
}

impl Serialize for Blacklist {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (user_id, group_id, code) = match &self.data {
            BlacklistData::User(u) => (Some(*u), None, None),
            BlacklistData::Group(g) => (None, Some(*g), None),
            BlacklistData::Custom(c) => (None, None, Some(c.code.clone())),
        };
        let intermediary = BlacklistIntermediary {
            blacklist_id: self.blacklist_id,
            reason: self.reason.clone(),
            kind: self.kind(),
            user_id,
            group_id,
            code,
        };
        intermediary.serialize(serializer)
    }
}
