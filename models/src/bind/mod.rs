mod asset;
mod custom;
mod group;
mod rank;
mod template;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;

use std::collections::HashMap;
use twilight_model::id::RoleId;

pub trait Bind {
    type BackupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind;
    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self;
}
