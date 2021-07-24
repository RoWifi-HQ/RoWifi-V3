use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    str::FromStr,
};

#[derive(Debug, Serialize_repr, Deserialize_repr, Eq, PartialEq, Copy, Clone)]
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

impl Display for BlacklistActionType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            BlacklistActionType::None => write!(f, "None"),
            BlacklistActionType::Kick => write!(f, "Kick"),
            BlacklistActionType::Ban => write!(f, "Ban"),
        }
    }
}

impl FromStr for BlacklistActionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Ok(BlacklistActionType::None),
            "kick" => Ok(BlacklistActionType::Kick),
            "ban" => Ok(BlacklistActionType::Ban),
            _ => Err(()),
        }
    }
}

impl Display for GuildType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            GuildType::Alpha => write!(f, "Alpha"),
            GuildType::Beta => write!(f, "Beta"),
            GuildType::Normal => write!(f, "Normal"),
        }
    }
}
