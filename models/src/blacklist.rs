use crate::rolang::{RoCommand, RoCommandUser};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Blacklist {
    pub id: String,
    pub reason: String,
    pub blacklist_type: BlacklistType,
}

#[derive(Clone)]
pub enum BlacklistType {
    Name(String),
    Group(i64),
    Custom(RoCommand),
}

impl Blacklist {
    pub fn evaluate(&self, user: &RoCommandUser) -> Result<bool, String> {
        match &self.blacklist_type {
            BlacklistType::Name(name) => Ok(user.user.roblox_id.to_string().eq(name)),
            BlacklistType::Group(id) => Ok(user.ranks.contains_key(id)),
            BlacklistType::Custom(cmd) => Ok(cmd.evaluate(user)?),
        }
    }
}

impl Serialize for Blacklist {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Blacklist", 3)?;
        state.serialize_field("_id", &self.id)?;
        state.serialize_field("Reason", &self.reason)?;
        let t = match self.blacklist_type {
            BlacklistType::Name(_) => 0,
            BlacklistType::Group(_) => 1,
            BlacklistType::Custom(_) => 2,
        };
        state.serialize_field("Type", &t)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Blacklist {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EncodedBlacklist {
            #[serde(rename = "_id")]
            pub id: String,

            #[serde(rename = "Reason")]
            pub reason: String,

            #[serde(rename = "Type")]
            pub blacklist_type: i8,
        }

        let input = EncodedBlacklist::deserialize(deserializer)?;
        let command = match input.blacklist_type {
            0 => BlacklistType::Name(input.id.clone()),
            1 => BlacklistType::Group(input.id.parse::<i64>().unwrap()),
            2 => BlacklistType::Custom(RoCommand::new(&input.id).unwrap()),
            _ => return Err(serde::de::Error::custom("Invalid blacklist type")),
        };

        Ok(Blacklist {
            id: input.id,
            reason: input.reason,
            blacklist_type: command,
        })
    }
}

impl fmt::Debug for BlacklistType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlacklistType::Name(_) => write!(f, "Name"),
            BlacklistType::Group(_) => write!(f, "Group"),
            BlacklistType::Custom(_) => write!(f, "Custom"),
        }
    }
}
