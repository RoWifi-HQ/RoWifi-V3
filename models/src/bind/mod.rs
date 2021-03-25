mod asset;
mod custom;
mod group;
mod rank;
mod template;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;

use std::{collections::HashMap, fmt::Debug};
use twilight_model::id::RoleId;

use crate::user::RoGuildUser;

pub trait Backup {
    type BackupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind;
    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self;
}

pub trait Bind: Send + Sync + Debug {
    fn nickname(&self, roblox_username: &str, user: &RoGuildUser, discord_nick: &str) -> String;
    fn priority(&self) -> i64;
}
