#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::if_not_else,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::cast_sign_loss
)]

use serde::Serializer;
use tokio_postgres::Row;

pub use twilight_model as discord;

pub mod analytics;
pub mod bind;
pub mod blacklist;
pub mod events;
pub mod guild;
pub mod id;
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

pub(crate) fn serialize_i64_as_string<S: Serializer>(x: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&x.to_string())
}

pub(crate) fn serialize_vec_as_string<S: Serializer>(x: &Vec<i64>, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(x.iter().map(|n| n.to_string()).collect::<Vec<_>>())
}

pub(crate) fn serialize_option_as_string<S: Serializer, T: ToString>(x: &Option<T>, serializer: S) -> Result<S::Ok, S::Error> {
    match x {
        Some(x) => serializer.serialize_some(&x.to_string()),
        None => serializer.serialize_none()
    }
}
