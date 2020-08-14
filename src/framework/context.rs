use std::sync::Arc;
use twilight::{
    http::Client as Http,
    cache::{InMemoryCache as Cache, twilight_cache_inmemory::model::*},
    model::id::*
};

use crate::utils::{Database, Roblox};
use crate::utils::error::RoError;

#[derive(Clone)]
pub struct Context {
    pub shard_id: u64,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub database: Arc<Database>,
    pub roblox: Arc<Roblox>
}

impl Context {
    pub fn new(shard_id: u64, http: Arc<Http>, cache: Arc<Cache>, database: Arc<Database>, roblox: Arc<Roblox>) -> Self {
        Self {
            shard_id,
            http,
            cache,
            database,
            roblox
        }
    } 

    pub async fn parse_member(&self, guild: GuildId, member_name: &str) -> Result<Option<Arc<CachedMember>>, RoError> {
        if let Ok(id) = member_name.parse::<u64>() {
            let member = self.get_member(guild, UserId(id)).await;
            if let Some(m) = member {
                return Ok(Some(m))
            }
            let member = self.http.guild_member(guild, UserId(id)).await?;
            match member {
                Some(m) => {
                    let res = self.cache.cache_member(guild, m).await;
                    return Ok(Some(res))
                },
                None => return Ok(None)
            } 
        }
        Ok(None)
    }

    pub async fn get_member(&self, guild: GuildId, id: UserId) -> Option<Arc<CachedMember>> {
        self.cache.member(guild, id).await.expect("The member cache got poisoned")
    }
}