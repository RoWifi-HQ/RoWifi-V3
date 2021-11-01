use std::{error::Error as StdError, fmt::{Display, Formatter, Result as FmtResult}};
use hyper::StatusCode;
use rowifi_redis::{PoolError, redis::RedisError};

#[derive(Debug)]
pub struct Error {
    pub(super) source: Option<Box<dyn StdError + Send + Sync>>,
    pub(super) kind: ErrorKind
}

impl Error {
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        todo!()
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_ref().map(|source| &**source as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    BuildingRequest,
    ChunkingResponse,
    Json {
        body: Vec<u8>
    },
    Redis,
    RequestError,
    Response {
        body: Vec<u8>,
        status: StatusCode,
        route: String
    },
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Redis
        }
    }
}

impl From<PoolError<RedisError>> for Error {
    fn from(err: PoolError<RedisError>) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Redis
        }
    }
}