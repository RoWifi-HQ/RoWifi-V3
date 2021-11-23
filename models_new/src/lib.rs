use tokio_postgres::Row;

pub mod bind;
pub mod roblox;
pub mod rolang;
pub mod user;

pub trait FromRow {
    fn from_row(row: Row) -> Result<Self, tokio_postgres::Error>
    where
        Self: Sized;
}
