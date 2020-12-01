mod asset;
mod custom;
mod group;
mod rank;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;

use std::collections::HashMap;
use twilight_model::id::RoleId;

pub trait Backup {
    type Bind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::Bind;
    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self;
}
