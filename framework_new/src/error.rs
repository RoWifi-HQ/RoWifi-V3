use patreon::PatreonError;
use roblox::RobloxError;
use rowifi_database::error::{DatabaseError, SerializationError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
use twilight_http::Error as DiscordHttpError;

use crate::arguments::ArgumentError;

#[derive(Debug)]
pub enum RoError {
    Argument,
    Database(DatabaseError),
    Roblox(RobloxError),
    Discord(DiscordHttpError),
    Patreon(PatreonError),
}

impl From<ArgumentError> for RoError {
    fn from(err: ArgumentError) -> Self {
        RoError::Argument
    }
}

impl Display for RoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RoError::Database(err) => write!(f, "Database Error - {:?}", err),
            RoError::Roblox(err) => write!(f, "Roblox Error - {:?}", err),
            RoError::Discord(err) => write!(f, "Discord Http Error - {}", err),
            RoError::Patreon(err) => write!(f, "Patreon Error - {}", err),
            RoError::Argument => write!(f, "Argument Error")
        }
    }
}

impl From<DatabaseError> for RoError {
    fn from(err: DatabaseError) -> Self {
        RoError::Database(err)
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

impl From<PatreonError> for RoError {
    fn from(err: PatreonError) -> Self {
        RoError::Patreon(err)
    }
}

impl From<SerializationError> for RoError {
    fn from(err: SerializationError) -> Self {
        RoError::Database(DatabaseError::Serialization(err))
    }
}

impl StdError for RoError {}

