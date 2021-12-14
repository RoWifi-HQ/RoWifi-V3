use tokio_postgres::Row;

pub use twilight_model as discord;

pub mod analytics;
pub mod bind;
pub mod roblox;
pub mod rolang;
pub mod user;
pub mod guild;
pub mod blacklist;
pub mod stats;
pub mod events;

pub trait FromRow {
    fn from_row(row: Row) -> Result<Self, tokio_postgres::Error>
    where
        Self: Sized;
}

impl FromRow for Row {
    fn from_row(row: Row) -> Result<Self, tokio_postgres::Error> {
        Ok(row)
    }
}
