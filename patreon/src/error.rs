use hyper::{http::Error as HttpError, Error as HyperError, StatusCode};
use serde_json::Error as SerdeError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum PatreonError {
    BuildingRequest(HttpError),
    Request(HyperError),
    Parsing(SerdeError),
    APIError(StatusCode),
}

impl Display for PatreonError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PatreonError::APIError(err) => write!(f, "API Error - {}", err),
            PatreonError::BuildingRequest(err) => write!(f, "Building Request Error - {}", err),
            PatreonError::Parsing(err) => write!(f, "Parsing Error - {}", err),
            PatreonError::Request(err) => write!(f, "Request Error - {}", err),
        }
    }
}

impl From<HttpError> for PatreonError {
    fn from(err: HttpError) -> Self {
        PatreonError::BuildingRequest(err)
    }
}

impl From<HyperError> for PatreonError {
    fn from(err: HyperError) -> Self {
        PatreonError::Request(err)
    }
}

impl From<SerdeError> for PatreonError {
    fn from(err: SerdeError) -> Self {
        PatreonError::Parsing(err)
    }
}

impl StdError for PatreonError {}
