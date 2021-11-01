use std::{error::Error as StdError, fmt::{Display, Formatter, Result as FmtResult}, time::Duration};
use twilight_http::{Error as DiscordHttpError, response::DeserializeBodyError};
use roblox::error::Error as RobloxError;
use rowifi_database::{DatabaseError, error::SerializationError as BsonSerializationError};
use patreon::PatreonError;

use crate::arguments::ArgumentError;

#[derive(Debug)]
pub struct RoError {
    pub(super) source: Option<Box<dyn StdError + Send + Sync>>,
    pub(super) kind: ErrorKind
}

impl RoError {
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn into_source(self) -> Option<Box<dyn StdError + Send + Sync>> {
        self.source
    }

    pub fn into_parts(self) -> (ErrorKind, Option<Box<dyn StdError + Send + Sync>>) {
        (self.kind, self.source)
    }
}

impl Display for RoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl StdError for RoError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|source| &**source as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Discord,
    Roblox,
    Database,
    Patreon,
    Command
}

#[derive(Debug)]
pub enum CommandError {
    Argument(ArgumentError),
    Cancelled,
    Message(MessageError),
    Timeout,
    Ratelimit(Duration),
    Other(String)
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl StdError for CommandError {}

#[derive(Debug)]
pub enum MessageError {

}

impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl StdError for MessageError {}

impl From<DiscordHttpError> for RoError {
    fn from(err: DiscordHttpError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Discord
        }
    }
}

impl From<DeserializeBodyError> for RoError {
    fn from(err: DeserializeBodyError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Discord
        }
    }
}

impl From<RobloxError> for RoError {
    fn from(err: RobloxError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Roblox
        }
    }
}

impl From<DatabaseError> for RoError {
    fn from(err: DatabaseError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Database
        }
    }
}

impl From<ArgumentError> for RoError {
    fn from(err: ArgumentError) -> Self {
        Self {
            source: Some(Box::new(CommandError::Argument(err))),
            kind: ErrorKind::Command
        }
    }
}

impl From<CommandError> for RoError {
    fn from(err: CommandError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Command
        }
    }
}

impl From<BsonSerializationError> for RoError {
    fn from(err: BsonSerializationError) -> Self {
        DatabaseError::Serialization(err).into()
    }
}

impl From<PatreonError> for RoError {
    fn from(err: PatreonError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Patreon
        }
    }
}
