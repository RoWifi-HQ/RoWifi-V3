#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::if_not_else,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::cast_sign_loss
)]

use tokio_postgres::Row;

pub use twilight_model as discord;

pub mod analytics;
pub mod bind;
pub mod blacklist;
pub mod events;
pub mod guild;
pub mod roblox;
pub mod rolang;
pub mod stats;
pub mod user;

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
