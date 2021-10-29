use hyper::{http::Error as HttpError, Error as HyperError, StatusCode};
use rowifi_redis::{redis::RedisError, PoolError};
use serde_json::Error as SerdeError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum Error {
    BuildingRequest(HttpError),
    Request(HyperError),
    Parsing(SerdeError),
    APIError(StatusCode, Vec<u8>),
    Redis(PoolError<RedisError>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::APIError(err, bytes) => write!(f, "API Error - {}, Body - {:?}", err, bytes),
            Error::BuildingRequest(err) => write!(f, "Building Request Error - {}", err),
            Error::Parsing(err) => write!(f, "Parsing Error - {}", err),
            Error::Request(err) => write!(f, "Request Error - {}", err),
            Error::Redis(err) => write!(f, "Redis Error - {}", err),
        }
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Error::BuildingRequest(err)
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Self {
        Error::Request(err)
    }
}

impl From<SerdeError> for Error {
    fn from(err: SerdeError) -> Self {
        Error::Parsing(err)
    }
}

impl From<RedisError> for Error {
    fn from(err: RedisError) -> Self {
        Error::Redis(PoolError::Backend(err))
    }
}

impl From<PoolError<RedisError>> for Error {
    fn from(err: PoolError<RedisError>) -> Self {
        Error::Redis(err)
    }
}

impl StdError for Error {}
