use deadpool_redis::{redis::RedisError, PoolError};
use hyper::StatusCode;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub struct Error {
    pub(super) source: Option<Box<dyn StdError + Send + Sync>>,
    pub(super) kind: ErrorKind,
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
        match &self.kind {
            ErrorKind::BuildingRequest => f.write_str("failed to build the request."),
            ErrorKind::ChunkingResponse => f.write_str("chunking the response failed."),
            ErrorKind::Json { body } => write!(f, "value failed to serialized: {:?}.", body),
            ErrorKind::Redis => f.write_str("error from redis occurred."),
            ErrorKind::RequestError => f.write_str("Parsing or sending the response failed"),
            ErrorKind::Response {
                body,
                status,
                route,
            } => write!(
                f,
                "response error: status code {}, route: {}, body: {:?}",
                status, route, body
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn StdError + 'static))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    BuildingRequest,
    ChunkingResponse,
    Json {
        body: Vec<u8>,
    },
    Redis,
    RequestError,
    Response {
        body: Vec<u8>,
        status: StatusCode,
        route: String,
    },
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Redis,
        }
    }
}

impl From<PoolError> for Error {
    fn from(err: PoolError) -> Self {
        Self {
            source: Some(Box::new(err)),
            kind: ErrorKind::Redis,
        }
    }
}
