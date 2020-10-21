use dashmap::{DashMap, DashSet};
use twilight_model::id::{ChannelId, GuildId, UserId};

#[derive(Default)]
pub struct Configuration {
    pub blocked_guilds: DashSet<GuildId>,
    pub blocked_users: DashSet<UserId>,
    pub disabled_channels: DashSet<ChannelId>,
    pub on_mention: String,
    pub default_prefix: String,
    pub owners: DashSet<UserId>,
    pub council: DashSet<UserId>,
    pub prefixes: DashMap<GuildId, String>,
}

pub struct BotConfig {
    pub cluster_id: u64,
    pub shards_per_cluster: u64,
    pub total_shards: u64,
}

impl Configuration {
    pub fn default_prefix(mut self, prefix: &str) -> Self {
        self.default_prefix = prefix.to_string();
        self
    }

    pub fn owners(mut self, user_ids: DashSet<UserId>) -> Self {
        self.owners = user_ids;
        self
    }

    pub fn on_mention(mut self, id_to_mention: UserId) -> Self {
        self.on_mention = id_to_mention.to_string();
        self
    }

    pub fn council(mut self, user_ids: DashSet<UserId>) -> Self {
        self.council = user_ids;
        self
    }
}
