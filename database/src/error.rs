pub use mongodb::{
    bson::{de::Error as DeserializationError, ser::Error as SerializationError},
    error::Error as MongoError,
};
use deadpool_redis::{redis::RedisError, PoolError};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[derive(Debug)]
pub enum DatabaseError {
    Serialization(SerializationError),
    Deserialization(DeserializationError),
    Mongo(Box<MongoError>),
    Redis(PoolError),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DatabaseError::Serialization(err) => write!(f, "Serialization Error - {}", err),
            DatabaseError::Deserialization(err) => write!(f, "Deserialization Error - {}", err),
            DatabaseError::Mongo(err) => write!(f, "Mongo Error - {}", err),
            DatabaseError::Redis(err) => write!(f, "Redis Error - {}", err),
        }
    }
}

impl From<SerializationError> for DatabaseError {
    fn from(err: SerializationError) -> Self {
        DatabaseError::Serialization(err)
    }
}

impl From<DeserializationError> for DatabaseError {
    fn from(err: DeserializationError) -> Self {
        DatabaseError::Deserialization(err)
    }
}

impl From<MongoError> for DatabaseError {
    fn from(err: MongoError) -> Self {
        DatabaseError::Mongo(Box::new(err))
    }
}

impl From<RedisError> for DatabaseError {
    fn from(err: RedisError) -> Self {
        DatabaseError::Redis(PoolError::Backend(err))
    }
}

impl From<PoolError> for DatabaseError {
    fn from(err: PoolError) -> Self {
        DatabaseError::Redis(err)
    }
}

impl StdError for DatabaseError {}
