use dashmap::{DashMap, DashSet};
use patreon::Client as Patreon;
use roblox::Client as Roblox;
use rowifi_cache::Cache;
use rowifi_database::Database;
use rowifi_models::stats::BotStats;
use std::{ops::Deref, sync::Arc};
use twilight_gateway::Cluster;
use twilight_http::Client as Http;
use twilight_model::id::{ChannelId, GuildId, UserId};
use twilight_standby::Standby;

pub struct BotContextRef {
    pub on_mention: String,
    pub prefixes: DashMap<GuildId, String>,
    pub default_prefix: String,
    pub disabled_channels: DashSet<ChannelId>,
    pub owners: DashSet<UserId>,
    pub http: Http,
    pub cache: Cache,
    pub cluster: Cluster,
    pub standby: Standby,
    pub database: Database,
    pub roblox: Roblox,
    pub patreon: Patreon,
    pub stats: Arc<BotStats>,
}

#[derive(Clone)]
pub struct BotContext(Arc<BotContextRef>);

#[derive(Clone)]
pub struct CommandContext {
    pub bot: BotContext,
}

impl BotContext {
    pub fn new(
        on_mention: String,
        default_prefix: String,
        owners: &[UserId],
        http: Http,
        cache: Cache,
        cluster: Cluster,
        standby: Standby,
        database: Database,
        roblox: Roblox,
        patreon: Patreon,
        stats: Arc<BotStats>,
    ) -> Self {
        let mut _owners = DashSet::new();
        _owners.extend(owners.iter().map(|u| u.to_owned()));
        Self {
            0: Arc::new(BotContextRef {
                on_mention,
                prefixes: DashMap::new(),
                default_prefix,
                disabled_channels: DashSet::new(),
                owners: _owners,
                http,
                cache,
                cluster,
                standby,
                database,
                roblox,
                patreon,
                stats,
            }),
        }
    }
}

impl Deref for BotContext {
    type Target = BotContextRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
