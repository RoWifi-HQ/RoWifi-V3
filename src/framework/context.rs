use std::sync::Arc;
use twilight_gateway::Cluster;
use twilight_http::Client as Http;
use twilight_model::id::*;
use twilight_standby::Standby;

use crate::cache::*;
use crate::models::{configuration::Configuration, stats::BotStats};
use crate::utils::error::RoError;
use crate::utils::{Database, Logger, Patreon, Roblox};

#[derive(Clone)]
pub struct Context {
    pub shard_id: u64,
    pub http: Http,
    pub cache: Cache,
    pub database: Database,
    pub roblox: Roblox,
    pub standby: Standby,
    pub cluster: Cluster,
    pub logger: Arc<Logger>,
    pub config: Arc<Configuration>,
    pub patreon: Patreon,
    pub stats: Arc<BotStats>,
}

impl Context {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        shard_id: u64,
        http: Http,
        cache: Cache,
        database: Database,
        roblox: Roblox,
        standby: Standby,
        cluster: Cluster,
        logger: Arc<Logger>,
        config: Arc<Configuration>,
        patreon: Patreon,
        stats: Arc<BotStats>,
    ) -> Self {
        Self {
            shard_id,
            http,
            cache,
            database,
            roblox,
            standby,
            cluster,
            logger,
            config,
            patreon,
            stats,
        }
    }

    pub async fn member(
        &self,
        guild_id: GuildId,
        user_id: impl Into<UserId>,
    ) -> Result<Option<Arc<CachedMember>>, RoError> {
        let user_id = user_id.into();

        if let Some(member) = self.cache.member(guild_id, user_id) {
            return Ok(Some(member));
        }
        match self.http.guild_member(guild_id, user_id).await? {
            Some(m) => {
                let cached = self.cache.cache_member(guild_id, m);
                Ok(Some(cached))
            }
            None => Ok(None),
        }
    }
}
