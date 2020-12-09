use bson::{de::Error as DeserializationError, ser::Error as SerializationError};
use mongodb::error::Error as MongoError;
use reqwest::Error as PatreonError;
use roblox::RobloxError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_http::Error as DiscordHttpError;

#[derive(Debug)]
pub enum RoError {
    Database(MongoError),
    Serialization(SerializationError),
    Deserialization(DeserializationError),
    Roblox(RobloxError),
    Discord(DiscordHttpError),
    Command(CommandError),
    Patreon(PatreonError),
}

#[derive(Debug)]
pub enum CommandError {
    NicknameTooLong(String),
    Blacklist(String),
    NoRoGuild,
    ParseArgument(String, String, String),
    Timeout,
}

impl Display for RoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RoError::Database(err) => write!(f, "Database Error - {}", err),
            RoError::Serialization(err) => write!(f, "Serialization Error - {}", err),
            RoError::Deserialization(err) => write!(f, "Deserialization Error - {}", err),
            RoError::Roblox(err) => write!(f, "Roblox Error - {:?}", err),
            RoError::Discord(err) => write!(f, "Discord Http Error - {}", err),
            RoError::Command(err) => write!(f, "Command Error - {:?}", err),
            RoError::Patreon(err) => write!(f, "Patreon Error - {}", err),
        }
    }
}

impl From<MongoError> for RoError {
    fn from(err: MongoError) -> Self {
        RoError::Database(err)
    }
}

impl From<SerializationError> for RoError {
    fn from(err: SerializationError) -> Self {
        RoError::Serialization(err)
    }
}

impl From<DeserializationError> for RoError {
    fn from(err: DeserializationError) -> Self {
        RoError::Deserialization(err)
    }
}

impl From<RobloxError> for RoError {
    fn from(err: RobloxError) -> Self {
        RoError::Roblox(err)
    }
}

impl From<DiscordHttpError> for RoError {
    fn from(err: DiscordHttpError) -> Self {
        RoError::Discord(err)
    }
}

impl From<CommandError> for RoError {
    fn from(err: CommandError) -> Self {
        RoError::Command(err)
    }
}

impl From<PatreonError> for RoError {
    fn from(err: PatreonError) -> Self {
        RoError::Patreon(err)
    }
}

impl StdError for RoError {}
