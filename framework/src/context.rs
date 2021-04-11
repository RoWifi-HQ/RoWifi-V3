use chacha20poly1305::ChaCha20Poly1305;
use dashmap::{DashMap, DashSet};
use itertools::Itertools;
use patreon::Client as Patreon;
use roblox::{
    models::id::{AssetId as RobloxAssetId, UserId as RobloxUserId},
    Client as Roblox,
};
use rowifi_cache::{Cache, CachedGuild, CachedMember};
use rowifi_database::Database;
use rowifi_models::{
    bind::Bind,
    guild::{BlacklistActionType, RoGuild},
    rolang::RoCommandUser,
    stats::BotStats,
    user::RoGuildUser,
};
use std::{
    borrow::ToOwned,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use twilight_gateway::Cluster;
use twilight_http::Client as Http;
use twilight_model::{
    channel::embed::Embed,
    id::{ChannelId, GuildId, InteractionId, RoleId, UserId, WebhookId},
    user::User,
};
use twilight_standby::Standby;
use twilight_util::link::webhook;

use crate::error::{CommandError, RoError};

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
    pub webhooks: HashMap<&'static str, (WebhookId, String)>,
    pub cluster_id: u64,
    pub total_shards: u64,
    pub shards_per_cluster: u64,
    pub cipher: ChaCha20Poly1305,
}

#[derive(Clone)]
pub struct BotContext(Arc<BotContextRef>);

#[derive(Clone)]
pub struct CommandContext {
    pub bot: BotContext,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub author: Arc<User>,
    pub interaction_id: Option<InteractionId>,
    pub interaction_token: Option<String>,
}

impl BotContext {
    #[allow(clippy::too_many_arguments)]
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
        webhooks: HashMap<&'static str, &str>,
        cluster_id: u64,
        total_shards: u64,
        shards_per_cluster: u64,
        cipher: ChaCha20Poly1305,
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
                cipher,
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
    pub async fn member(
        &self,
        guild_id: GuildId,
        user_id: impl Into<UserId>,
    ) -> Result<Option<Arc<CachedMember>>, RoError> {
        let user_id = user_id.into();

        if let Some(member) = self.bot.cache.member(guild_id, user_id) {
            return Ok(Some(member));
        }
        match self.bot.http.guild_member(guild_id, user_id).await? {
            Some(m) => {
                let cached = self.bot.cache.cache_member(guild_id, m);
                Ok(Some(cached))
            }
            None => Ok(None),
        }
    }

    pub async fn update_user(
        &self,
        member: Arc<CachedMember>,
        user: &RoGuildUser,
        server: &CachedGuild,
        guild: &RoGuild,
        guild_roles: &HashSet<RoleId>,
    ) -> Result<(Vec<RoleId>, Vec<RoleId>, String), RoError> {
        self.bot
            .update_user(member, user, server, guild, guild_roles)
            .await
    }

    pub async fn get_linked_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Option<RoGuildUser>, RoError> {
        self.bot.get_linked_user(user_id, guild_id).await
    }

    pub async fn log_guild(&self, guild_id: GuildId, embed: Embed) {
        self.bot.log_guild(guild_id, embed).await
    }

    pub async fn log_debug(&self, embed: Embed) {
        self.bot.log_debug(embed).await
    }

    pub async fn log_error(&self, text: &str) {
        self.bot.log_error(text).await
    }

    pub async fn log_premium(&self, text: &str) {
        self.bot.log_premium(text).await
    }
}

impl BotContext {
    #[allow(clippy::needless_collect)]
    pub async fn update_user(
        &self,
        member: Arc<CachedMember>,
        user: &RoGuildUser,
        server: &CachedGuild,
        guild: &RoGuild,
        guild_roles: &HashSet<RoleId>,
    ) -> Result<(Vec<RoleId>, Vec<RoleId>, String), RoError> {
        let mut added_roles = Vec::<RoleId>::new();
        let mut removed_roles = Vec::<RoleId>::new();

        let verification_role = RoleId(guild.verification_role as u64);
        if guild_roles.get(&verification_role).is_some()
            && member.roles.contains(&verification_role)
        {
            removed_roles.push(verification_role);
        }

        let verified_role = RoleId(guild.verified_role as u64);
        if guild_roles.get(&verified_role).is_some() && !member.roles.contains(&verified_role) {
            added_roles.push(verified_role);
        }

        let user_id = RobloxUserId(user.roblox_id as u64);
        let user_roles = self
            .roblox
            .get_user_roles(user_id)
            .await?
            .iter()
            .map(|r| (r.group.id.0 as i64, r.role.rank as i64))
            .collect::<HashMap<_, _>>();
        let roblox_user = self.roblox.get_user(user_id).await?;
        let command_user = RoCommandUser {
            user: &user,
            roles: &member.roles,
            ranks: &user_roles,
            username: &roblox_user.name,
        };

        if !guild.blacklists.is_empty() {
            let success = guild
                .blacklists
                .iter()
                .find(|b| b.evaluate(&command_user).unwrap());
            if let Some(success) = success {
                match guild.settings.blacklist_action {
                    BlacklistActionType::None => {}
                    BlacklistActionType::Kick => {
                        let _ = self
                            .http
                            .remove_guild_member(server.id, member.user.id)
                            .await;
                    }
                    BlacklistActionType::Ban => {
                        let _ = self.http.create_ban(server.id, member.user.id).await;
                    }
                };
                return Err(RoError::Command(CommandError::Blacklist(
                    success.reason.clone(),
                )));
            }
        }

        let mut nick_bind: Option<&dyn Bind> = None;
        let mut roles_to_add = Vec::new();

        for r in &guild.rankbinds {
            let to_add = match user_roles.get(&r.group_id) {
                Some(rank_id) => *rank_id == r.rank_id as i64,
                None => r.rank_id == 0,
            };
            if to_add {
                if let Some(highest) = nick_bind {
                    if highest.priority() < r.priority() {
                        nick_bind = Some(r);
                    }
                } else {
                    nick_bind = Some(r);
                }
                roles_to_add.extend(r.discord_roles.iter().cloned());
            }
        }

        for g in &guild.groupbinds {
            if user_roles.contains_key(&g.group_id) {
                if let Some(highest) = nick_bind {
                    if highest.priority() < g.priority() {
                        nick_bind = Some(g);
                    }
                } else {
                    nick_bind = Some(g);
                }
                roles_to_add.extend(g.discord_roles.iter().cloned());
            }
        }

        for c in &guild.custombinds {
            if c.command.evaluate(&command_user).unwrap() {
                if let Some(highest) = nick_bind {
                    if highest.priority() < c.priority() {
                        nick_bind = Some(c);
                    }
                } else {
                    nick_bind = Some(c);
                }
                roles_to_add.extend(c.discord_roles.iter().cloned());
            }
        }

        for a in &guild.assetbinds {
            if self
                .roblox
                .get_asset(
                    user_id,
                    RobloxAssetId(a.id as u64),
                    &a.asset_type.to_string(),
                )
                .await?
                .is_some()
            {
                if let Some(highest) = nick_bind {
                    if highest.priority() < a.priority() {
                        nick_bind = Some(a);
                    }
                } else {
                    nick_bind = Some(a);
                }
                roles_to_add.extend(a.discord_roles.iter().cloned());
            }
        }

        for bind_role in &guild.all_roles {
            let r = RoleId(*bind_role as u64);
            if guild_roles.get(&r).is_some() {
                if roles_to_add.contains(&bind_role) {
                    if !member.roles.contains(&r) {
                        added_roles.push(r);
                    }
                } else if member.roles.contains(&r) {
                    removed_roles.push(r);
                }
            }
        }

        let discord_nick = member
            .nick
            .as_ref()
            .map_or_else(|| member.user.name.as_str(), |s| s.as_str());
        let nick_bypass = match server.nickname_bypass {
            Some(n) => member.roles.contains(&n),
            None => false,
        };
        let nickname = if nick_bypass {
            discord_nick.to_string()
        } else {
            nick_bind.map_or_else(
                || roblox_user.name.to_string(),
                |nick_bind| nick_bind.nickname(&roblox_user.name, &user, discord_nick),
            )
        };

        if nickname.len() > 32 {
            return Err(RoError::Command(CommandError::Miscellanous(format!(
                "The supposed nickname {} was found to be more than 32 characters",
                nickname
            ))));
        }

        let update = self.http.update_guild_member(server.id, member.user.id);
        let role_changes = !added_roles.is_empty() || !removed_roles.is_empty();
        let mut roles = member.roles.clone();
        roles.extend_from_slice(&added_roles);
        roles.retain(|r| !removed_roles.contains(r));
        roles = roles.into_iter().unique().collect::<Vec<RoleId>>();

        let nick_changes = nickname != discord_nick;

        if role_changes || nick_changes {
            update.roles(roles).nick(nickname.clone()).unwrap().await?;
        }

        Ok((added_roles, removed_roles, nickname))
    }

    #[allow(clippy::cast_possible_wrap)]
    pub async fn get_linked_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Option<RoGuildUser>, RoError> {
        let mut linked_user = self.database.get_linked_user(user_id.0, guild_id.0).await?;
        if linked_user.is_none() {
            let user = self.database.get_user(user_id.0).await?;
            if let Some(user) = user {
                linked_user = Some(RoGuildUser {
                    guild_id: guild_id.0 as i64,
                    discord_id: user.discord_id,
                    roblox_id: user.roblox_id,
                });
            }
        }
        Ok(linked_user)
    }

    pub async fn log_debug(&self, embed: Embed) {
        let (id, token) = self.webhooks.get("debug").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .embeds(vec![embed])
            .await;
    }

    pub async fn log_error(&self, text: &str) {
        let (id, token) = self.webhooks.get("error").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .content(text.to_string())
            .await;
    }

    pub async fn log_premium(&self, text: &str) {
        let (id, token) = self.webhooks.get("premium").unwrap();
        let _ = self
            .http
            .execute_webhook(*id, token)
            .content(text.to_string())
            .await;
    }

    pub async fn log_guild(&self, guild_id: GuildId, embed: Embed) {
        let log_channel = self.cache.guild(guild_id).and_then(|g| g.log_channel);
        if let Some(channel_id) = log_channel {
            let _ = self
                .http
                .create_message(channel_id)
                .embed(embed)
                .unwrap()
                .await;
        }
    }
}
