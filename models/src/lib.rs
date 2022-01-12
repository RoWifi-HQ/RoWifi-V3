#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::if_not_else,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
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

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn serialize_i64_as_string<S: Serializer>(
    x: &i64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&x.to_string())
}

#[allow(clippy::ptr_arg)]
pub(crate) fn serialize_vec_as_string<S: Serializer>(
    x: &Vec<i64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(x.iter().map(ToString::to_string).collect::<Vec<_>>())
}
