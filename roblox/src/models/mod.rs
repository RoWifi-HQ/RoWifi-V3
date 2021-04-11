pub mod asset;
pub mod group;
pub mod id;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct VecWrapper<T> {
    pub data: Vec<T>,
}
