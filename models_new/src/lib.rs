use tokio_postgres::Row;

pub mod bind;

pub trait FromRow {
    fn from_row(row: Row) -> Result<Self, tokio_postgres::Error>
    where
        Self: Sized;
}
