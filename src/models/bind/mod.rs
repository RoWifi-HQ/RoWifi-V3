mod asset;
mod custom;
mod group;
mod rank;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;

use std::{collections::HashMap, sync::Arc};
use twilight_model::id::RoleId;

use crate::cache::CachedRole;

pub trait Backup {
    type Bind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind;
    fn from_backup(bind: &Self::Bind, roles: &HashMap<String, RoleId>) -> Self;
}