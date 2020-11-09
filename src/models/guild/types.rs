use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::fmt;

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq, Clone)]
#[repr(i8)]
pub enum GuildType {
    Alpha,
    Beta,
    Normal,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Copy, Clone)]
#[repr(i8)]
pub enum BlacklistActionType {
    None,
    Kick,
    Ban,
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

impl Default for GuildType {
    fn default() -> Self {
        GuildType::Normal
    }
}

impl Default for BlacklistActionType {
    fn default() -> Self {
        BlacklistActionType::None
    }
}

impl fmt::Display for BlacklistActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlacklistActionType::None => write!(f, "None"),
            BlacklistActionType::Kick => write!(f, "Kick"),
            BlacklistActionType::Ban => write!(f, "Ban"),
        }
    }
}

impl fmt::Display for GuildType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuildType::Alpha => write!(f, "Alpha"),
            GuildType::Beta => write!(f, "Beta"),
            GuildType::Normal => write!(f, "Normal"),
        }
    }
}
