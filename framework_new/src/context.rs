use std::{ops::Deref, sync::Arc};
use dashmap::{DashMap, DashSet};
use twilight_http::Client as Http;
use twilight_model::id::{ChannelId, GuildId};

#[derive(Debug, Default)]
pub struct BotContextRef {
    pub on_mention: String,
    pub prefixes: DashMap<GuildId, String>,
    pub default_prefix: String,
    pub disabled_channels: DashSet<ChannelId>
}

pub struct BotContext(Arc<BotContextRef>);

#[derive(Clone)]
pub struct CommandContext {
    pub http: Http
}

impl BotContext {
    pub fn new(on_mention: String, default_prefix: String) -> Self {
        Self {
            0: Arc::new(BotContextRef {
                on_mention,
                prefixes: DashMap::new(),
                default_prefix,
                disabled_channels: DashSet::new()
            })
        }
    }
}

impl Deref for BotContext {
    type Target = BotContextRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}