use hyper::{http::Error as HttpError, Error as HyperError};
use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum Error {
    BuildingRequest(HttpError),
    Request(HyperError),
    Parsing(SerdeError),
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
