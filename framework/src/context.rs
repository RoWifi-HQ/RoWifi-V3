use dashmap::{DashMap, DashSet};
use patreon::Client as Patreon;
use roblox::Client as Roblox;
use rowifi_cache::{Cache, CachedGuild, CachedMember};
use rowifi_database::Database;
use rowifi_models::{
    discord::{
        application::interaction::application_command::CommandInteractionDataResolved,
        channel::embed::Embed,
        id::{ChannelId, InteractionId, MessageId, RoleId, UserId, WebhookId},
        user::User,
    },
    id::GuildId,
    stats::BotStats,
};
use std::{
    borrow::ToOwned,
    collections::HashMap,
    ops::Deref,
    sync::{atomic::AtomicBool, Arc},
};
use twilight_gateway::Cluster;
use twilight_http::{error::ErrorType as DiscordErrorType, Client as Http};
use twilight_standby::Standby;
use twilight_util::link::webhook;

use crate::{error::RoError, respond::Responder};

pub struct BotContextRef {
    // Config Items
    /// The mention prefix of the bot.
    /// TODO: Find an alternative way of checking mention prefixes rather storing this string
    pub on_mention: String,
    /// The map containing prefixes of all servers
    pub prefixes: DashMap<GuildId, String>,
    /// The default prefix of the bot
    pub default_prefix: String,
    /// The set holding all channels where the bot is configured not to respond
    pub disabled_channels: DashSet<ChannelId>,
    /// The set containing all owners of the bot
    pub owners: DashSet<UserId>,
    /// The map containing the set of admin roles for all servers
    pub admin_roles: DashMap<GuildId, Vec<RoleId>>,
    /// The map containing the set of trainer roles for all servers
    pub trainer_roles: DashMap<GuildId, Vec<RoleId>>,
    /// The map containing the set of bypass roles for all servers
    pub bypass_roles: DashMap<GuildId, Vec<RoleId>>,
    /// The map containing the set of nickname roles for all servers
    pub nickname_bypass_roles: DashMap<GuildId, Vec<RoleId>>,
    /// The map containing log channels of all servers
    pub log_channels: DashMap<GuildId, ChannelId>,
    /// The array containing the message ids wit active components
    pub ignore_message_components: DashSet<MessageId>,

    // Twilight Components
    /// The module used to make requests to discord
    pub http: Arc<Http>,
    /// The module holding all discord structs in-memory
    pub cache: Cache,
    /// The module handling the gateway
    pub cluster: Arc<Cluster>,
    /// The module for waiting for certain events within commands
    pub standby: Standby,
    /// The pre-configured webhooks that we write to for logging purposes
    pub webhooks: HashMap<&'static str, (WebhookId, String)>,

    // RoWifi Modules
    /// The module handling all connections to Mongo
    pub database: Database,
    /// The Roblox API Wrapper struct
    pub roblox: Roblox,
    /// The Patreon API Wrapper struct
    pub patreon: Patreon,
    /// The module collecting events data. This is an Arc because we distribute this across multiple components
    pub stats: Arc<BotStats>,

    // Cluster Config
    pub cluster_id: u64,
    pub total_shards: u64,
    pub shards_per_cluster: u64,
}

/// The struct that contains all bot config fields. We use an internally arced struct here
/// since we have a lot of fields and we don't want to be bogged down by the clone function
#[derive(Clone)]
pub struct BotContext(Arc<BotContextRef>);

#[derive(Clone)]
pub struct CommandContext {
    /// The struct holding all bot config fields
    pub bot: BotContext,
    /// The channel id from which the interaction or message came from
    pub channel_id: ChannelId,
    /// The guild id from which the interaction or message came from.
    /// We keep this an option in case we ever support DM commands
    pub guild_id: Option<GuildId>,
    /// The struct containing fields of the author
    pub author: Arc<User>,
    /// The id of the original message invoked by the user
    pub message_id: Option<MessageId>,
    /// The id of the interaction
    pub interaction_id: Option<InteractionId>,
    /// The token of the interaction. This is used to make followups or edit the original response
    pub interaction_token: Option<String>,
    /// Bool whether callback has been invoked
    pub callback_invoked: Arc<AtomicBool>,
    /// The resolved data sent in an interaction
    pub resolved: Option<CommandInteractionDataResolved>,
}

impl BotContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        on_mention: String,
        default_prefix: String,
        owners: &[UserId],
        http: Arc<Http>,
        cache: Cache,
        cluster: Arc<Cluster>,
        standby: Standby,
        database: Database,
        roblox: Roblox,
        patreon: Patreon,
        stats: Arc<BotStats>,
        webhooks: HashMap<&'static str, &str>,
        cluster_id: u64,
        total_shards: u64,
        shards_per_cluster: u64,
    ) -> Self {
        let mut owners_set = DashSet::new();
        owners_set.extend(owners.iter().map(ToOwned::to_owned));

        let mut webhooks_map = HashMap::new();
        for (name, url) in webhooks {
            let (id, token) = webhook::parse(url).unwrap();
            webhooks_map.insert(name, (id, token.unwrap().to_owned()));
        }
        Self {
            0: Arc::new(BotContextRef {
                on_mention,
                prefixes: DashMap::new(),
                default_prefix,
                disabled_channels: DashSet::new(),
                owners: owners_set,
                admin_roles: DashMap::new(),
                trainer_roles: DashMap::new(),
                bypass_roles: DashMap::new(),
                nickname_bypass_roles: DashMap::new(),
                log_channels: DashMap::new(),
                ignore_message_components: DashSet::new(),
                http,
                cache,
                cluster,
                standby,
                database,
                roblox,
                patreon,
                stats,
                webhooks: webhooks_map,
                cluster_id,
                total_shards,
                shards_per_cluster,
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

impl CommandContext {
    pub fn respond(&self) -> Responder {
        Responder::new(self)
    }

    pub async fn member(
        &self,
        guild_id: GuildId,
        user_id: impl Into<UserId>,
    ) -> Result<Option<Arc<CachedMember>>, RoError> {
        let user_id = user_id.into();

        if let Some(member) = self.bot.cache.member(guild_id, user_id) {
            return Ok(Some(member));
        }
        let res = self.bot.http.guild_member(guild_id.0, user_id).exec().await;
        match res {
            Err(e) => {
                if let DiscordErrorType::Response {
                    body: _,
                    error: _,
                    status,
                } = e.kind()
                {
                    if *status == 404 {
                        return Ok(None);
                    }
                }
                Err(e.into())
            }
            Ok(res) => {
                let member = res.model().await?;
                let cached = self.bot.cache.cache_member(guild_id, member);
                Ok(Some(cached))
            }
        }
    }

    // pub async fn get_linked_user(
    //     &self,
    //     user_id: UserId,
    //     guild_id: GuildId,
    // ) -> Result<Option<RoGuildUser>, RoError> {
    //     self.bot.get_linked_user(user_id, guild_id).await
    // }

    pub async fn log_guild(&self, guild_id: GuildId, embed: Embed) {
        self.bot.log_guild(guild_id, embed).await;
    }

    pub async fn log_debug(&self, embed: Embed) {
        self.bot.log_debug(embed).await;
    }

    pub async fn log_error(&self, text: &str) {
        self.bot.log_error(text).await;
    }

    pub async fn log_premium(&self, text: &str) {
        self.bot.log_premium(text).await;
    }
}

impl BotContext {
    pub async fn log_debug(&self, embed: Embed) {
        let (id, token) = self.webhooks.get("debug").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .embeds(&[embed])
            .exec()
            .await;
    }

    pub async fn log_error(&self, text: &str) {
        let (id, token) = self.webhooks.get("error").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .content(text)
            .exec()
            .await;
    }

    pub async fn log_premium(&self, text: &str) {
        let (id, token) = self.webhooks.get("premium").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .content(text)
            .exec()
            .await;
    }

    pub async fn log_guild(&self, guild_id: GuildId, embed: Embed) {
        if let Some(log_channel) = self.log_channels.get(&guild_id) {
            let _ = self
                .http
                .create_message(*log_channel)
                .embeds(&[embed])
                .unwrap()
                .exec()
                .await;
        } else {
            let log_channel = self.cache.guild(guild_id).and_then(|g| g.log_channel);
            if let Some(channel_id) = log_channel {
                let _ = self
                    .http
                    .create_message(channel_id)
                    .embeds(&[embed])
                    .unwrap()
                    .exec()
                    .await;
            }
        }
    }

    pub fn has_bypass_role(&self, server: &CachedGuild, member: &CachedMember) -> bool {
        if let Some(bypass_role) = server.bypass_role {
            if member.roles.contains(&bypass_role) {
                return true;
            }
        }

        if let Some(bypass_roles) = self.bypass_roles.get(&server.id) {
            for bypass_role in bypass_roles.value() {
                if member.roles.contains(bypass_role) {
                    return true;
                }
            }
        }

        false
    }

    pub fn has_nickname_bypass(&self, server: &CachedGuild, member: &CachedMember) -> bool {
        if let Some(nickname_bypass) = server.nickname_bypass {
            if member.roles.contains(&nickname_bypass) {
                return true;
            }
        }

        if let Some(nickname_bypass) = self.nickname_bypass_roles.get(&server.id) {
            for nb in nickname_bypass.value() {
                if member.roles.contains(nb) {
                    return true;
                }
            }
        }

        false
    }
}
