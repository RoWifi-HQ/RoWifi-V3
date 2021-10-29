mod asset;
mod custom;
mod group;
mod rank;
mod template;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;
pub use template::*;

use std::{collections::HashMap, fmt::Debug};
use twilight_model::id::RoleId;

use crate::roblox::user::PartialUser as RobloxUser;
use crate::user::RoGuildUser;

pub trait Backup {
    type BackupBind;

    fn to_backup(&self, roles: &HashMap<RoleId, String>) -> Self::BackupBind;
    fn from_backup(bind: &Self::BackupBind, roles: &HashMap<String, RoleId>) -> Self;
}

pub trait Bind: Send + Sync + Debug {
    fn nickname(
        &self,
        roblox_user: &RobloxUser,
        user: &RoGuildUser,
        discord_username: &str,
        discord_nick: &Option<String>,
    ) -> String;
    fn priority(&self) -> i64;
}
