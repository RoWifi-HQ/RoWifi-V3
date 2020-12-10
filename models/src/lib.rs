#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::if_not_else,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::match_on_vec_items,
    clippy::map_err_ignore
)]

pub mod analytics;
pub mod bind;
pub mod blacklist;
pub mod events;
pub mod guild;
pub mod rolang;
pub mod stats;
pub mod user;
