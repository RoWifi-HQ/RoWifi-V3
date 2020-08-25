use std::sync::Arc;
use twilight::{
    http::Client as Http,
    model::id::*,
    standby::Standby
};

use crate::cache::*;
use crate::utils::{Database, Roblox};
use crate::utils::error::RoError;

#[derive(Clone)]
pub struct Context {
    pub shard_id: u64,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub database: Arc<Database>,
    pub roblox: Arc<Roblox>,
    pub standby: Arc<Standby>
}

impl Context {
    pub fn new(shard_id: u64, http: Arc<Http>, cache: Arc<Cache>, database: Arc<Database>, roblox: Arc<Roblox>, standby: Arc<Standby>) -> Self {
        Self {
            shard_id,
            http,
            cache,
            database,
            roblox,
            standby
        }
    }

    pub async fn member(&self, guild_id: GuildId, user_id: impl Into<UserId>) -> Result<Option<Arc<CachedMember>>, RoError> {
        let user_id = user_id.into();
        
        if let Some(member) = self.cache.member(guild_id, user_id) {
            return Ok(Some(member))
        }
        match self.http.guild_member(guild_id, user_id).await? {
            Some(m) => {
                let cached = self.cache.cache_member(guild_id, m);
                Ok(Some(cached))
            },
            None => Ok(None)
        }
    }
}