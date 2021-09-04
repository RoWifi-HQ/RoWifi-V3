use patreon::PatreonError;
use roblox::error::Error as RobloxError;
use rowifi_database::error::{DatabaseError, SerializationError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    time::Duration,
};
use twilight_http::{Error as DiscordHttpError, response::DeserializeBodyError};

use crate::arguments::ArgumentError;

#[derive(Debug)]
pub enum CommandError {
    Timeout,
    Blacklist(String),
    Miscellanous(String),
    Ratelimit(Duration),
}

#[derive(Debug)]
pub enum CommonError {
    UnknownMember,
}

#[derive(Debug)]
pub enum RoError {
    Argument(ArgumentError),
    Database(DatabaseError),
    Roblox(RobloxError),
    Discord(DiscordHttpError),
    Patreon(PatreonError),
    Command(CommandError),
    Common(CommonError),
    NoOp,
}

impl From<ArgumentError> for RoError {
    fn from(err: ArgumentError) -> Self {
        RoError::Argument(err)
    }
}

impl Display for RoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RoError::Database(err) => write!(f, "Database Error - {:?}", err),
            RoError::Roblox(err) => write!(f, "Roblox Error - {:?}", err),
            RoError::Discord(err) => write!(f, "Discord Http Error - {}", err),
            RoError::Patreon(err) => write!(f, "Patreon Error - {}", err),
            RoError::Argument(err) => write!(f, "Argument Error - {:?}", err),
            RoError::Command(err) => write!(f, "Command Error - {:?}", err),
            RoError::Common(err) => write!(f, "Common Error - {:?}", err),
            RoError::NoOp => write!(f, "NoOp error"),
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

impl From<CommandError> for RoError {
    fn from(err: CommandError) -> Self {
        RoError::Command(err)
    }
}

impl From<CommonError> for RoError {
    fn from(err: CommonError) -> Self {
        RoError::Common(err)
    }
}

impl From<DeserializeBodyError> for RoError {
    fn from(_: DeserializeBodyError) -> Self {
        RoError::NoOp
    }
}

impl StdError for RoError {}
