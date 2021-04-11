use hyper::{http::Error as HttpError, Error as HyperError, StatusCode};
use rowifi_redis::{PoolError, redis::RedisError};
use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum Error {
    BuildingRequest(HttpError),
    Request(HyperError),
    Parsing(SerdeError),
    APIError(StatusCode),
    Redis(PoolError<RedisError>)
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
