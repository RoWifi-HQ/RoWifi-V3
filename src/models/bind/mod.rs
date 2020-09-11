mod asset;
mod custom;
mod group;
mod rank;

pub use asset::*;
pub use custom::*;
pub use group::*;
pub use rank::*;

use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use twilight_model::id::{RoleId, GuildId};

use crate::cache::CachedRole;
use crate::framework::context::Context;

#[async_trait]
pub trait Backup {
    type Bind;

    fn to_backup(&self, roles: &HashMap<RoleId, Arc<CachedRole>>) -> Self::Bind;
    async fn from_backup(ctx: &Context, guild_id: GuildId, bind: Self::Bind, roles: &Vec<Arc<CachedRole>>) -> Self;
}