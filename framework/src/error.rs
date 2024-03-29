use patreon::PatreonError;
use roblox::error::Error as RobloxError;
use rowifi_database::error::{DatabaseError, PostgresError};
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    time::Duration,
};
use twilight_embed_builder::EmbedError;
use twilight_http::{response::DeserializeBodyError, Error as DiscordHttpError};
use twilight_validate::message::MessageValidationError;

use crate::arguments::ArgumentError;

#[derive(Debug)]
pub struct RoError {
    pub(super) source: Option<Box<dyn StdError + Send + Sync>>,
    pub(super) kind: ErrorKind,
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

    pub fn parts(&self) -> (&ErrorKind, &Option<Box<dyn StdError + Send + Sync>>) {
        (&self.kind, &self.source)
    }
}

impl Display for RoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.kind {
            ErrorKind::Command => f.write_str("command error: ")?,
            ErrorKind::Database => f.write_str("database error: ")?,
            ErrorKind::Discord => f.write_str("discord error: ")?,
            ErrorKind::Patreon => f.write_str("patreon error: ")?,
            ErrorKind::Roblox => f.write_str("roblox error: ")?,
        };
        match self.source() {
            Some(err) => Display::fmt(&err, f),
            None => f.write_str(""),
        }
    }
}

impl StdError for RoError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Discord,
    Roblox,
    Database,
    Patreon,
    Command,
}

#[derive(Debug)]
pub enum CommandError {
    Argument(ArgumentError),
    Cancelled,
    Message(MessageError),
    Timeout,
    Ratelimit(Duration),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Argument(arg_err) => write!(f, "argument error: {:?}", arg_err),
            Self::Cancelled => write!(f, "command cancelled."),
            Self::Message(msg_err) => write!(f, "message error: {}", msg_err),
            Self::Timeout => write!(f, "command timed out."),
            Self::Ratelimit(d) => write!(f, "command ratelimited: {}", d.as_secs()),
        }
    }
}

impl StdError for CommandError {}

#[derive(Debug)]
pub enum MessageError {
    Message(MessageValidationError),
    Embed(EmbedError),
}

impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Message(err) => Debug::fmt(&err, f),
            Self::Embed(err) => Debug::fmt(&err, f),
        }
    }
}

impl StdError for MessageError {}

impl From<DiscordHttpError> for RoError {
    fn from(err: DiscordHttpError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Discord,
        }
    }
}

impl From<DeserializeBodyError> for RoError {
    fn from(err: DeserializeBodyError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Discord,
        }
    }
}

impl From<RobloxError> for RoError {
    fn from(err: RobloxError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Roblox,
        }
    }
}

impl From<PostgresError> for RoError {
    fn from(err: PostgresError) -> Self {
        DatabaseError::from(err).into()
    }
}

impl From<DatabaseError> for RoError {
    fn from(err: DatabaseError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Database,
        }
    }
}

impl From<ArgumentError> for RoError {
    fn from(err: ArgumentError) -> Self {
        Self {
            source: Some(Box::new(CommandError::Argument(err))),
            kind: ErrorKind::Command,
        }
    }
}

impl From<CommandError> for RoError {
    fn from(err: CommandError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Command,
        }
    }
}

impl From<PatreonError> for RoError {
    fn from(err: PatreonError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Patreon,
        }
    }
}

impl From<MessageError> for RoError {
    fn from(err: MessageError) -> Self {
        Self {
            source: Some(Box::new(CommandError::Message(err))),
            kind: ErrorKind::Command,
        }
    }
}

impl From<MessageValidationError> for MessageError {
    fn from(err: MessageValidationError) -> Self {
        MessageError::Message(err)
    }
}

impl From<MessageValidationError> for RoError {
    fn from(err: MessageValidationError) -> Self {
        Self {
            source: Some(Box::new(CommandError::Message(MessageError::Message(err)))),
            kind: ErrorKind::Command,
        }
    }
}

impl From<EmbedError> for MessageError {
    fn from(err: EmbedError) -> Self {
        MessageError::Embed(err)
    }
}

impl From<EmbedError> for RoError {
    fn from(err: EmbedError) -> Self {
        Self {
            source: Some(Box::new(CommandError::Message(MessageError::Embed(err)))),
            kind: ErrorKind::Command,
        }
    }
}
