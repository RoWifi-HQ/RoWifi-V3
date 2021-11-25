use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

pub use deadpool_postgres::PoolError;
pub use tokio_postgres::Error as PostgresError;

#[derive(Debug)]
pub enum DatabaseError {
    Postgres(PoolError),
}

impl From<PoolError> for DatabaseError {
    fn from(err: PoolError) -> Self {
        DatabaseError::Postgres(err)
    }
}

impl From<PostgresError> for DatabaseError {
    fn from(err: PostgresError) -> Self {
        PoolError::from(err).into()
    }
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Postgres(err) => Display::fmt(err, f),
        }
    }
}

impl StdError for DatabaseError {}
