use dashmap::{DashMap, DashSet};
use itertools::Itertools;
use patreon::Client as Patreon;
use roblox::Client as Roblox;
use rowifi_cache::{Cache, CachedGuild, CachedMember};
use rowifi_database::Database;
use rowifi_models::{
    guild::{BlacklistActionType, RoGuild},
    rolang::RoCommandUser,
    stats::BotStats,
    user::RoUser,
};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};
use twilight_gateway::Cluster;
use twilight_http::Client as Http;
use twilight_model::{
    channel::embed::Embed,
    id::{ChannelId, GuildId, RoleId, UserId, WebhookId},
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
}

#[derive(Clone)]
pub struct BotContext(Arc<BotContextRef>);

#[derive(Clone)]
pub struct CommandContext {
    pub bot: BotContext,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
    pub author_id: UserId,
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
        webhooks: HashMap<&'static str, &str>,
        cluster_id: u64,
    ) -> Self {
        let mut _owners = DashSet::new();
        _owners.extend(owners.iter().map(|u| u.to_owned()));

        let mut _webhooks = HashMap::new();
        for (name, url) in webhooks {
            let (id, token) = webhook::parse(url).unwrap();
            _webhooks.insert(name, (id, token.unwrap().to_owned()));
        }
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
                webhooks: _webhooks,
                cluster_id,
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
        user: &RoUser,
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

        let user_roles = self.bot.roblox.get_user_roles(user.roblox_id).await?;
        let username = self.bot.roblox.get_username_from_id(user.roblox_id).await?;
        let command_user = RoCommandUser {
            user: &user,
            roles: &member.roles,
            ranks: &user_roles,
            username: &username,
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
                            .bot
                            .http
                            .remove_guild_member(server.id, member.user.id)
                            .await;
                    }
                    BlacklistActionType::Ban => {
                        let _ = self.bot.http.create_ban(server.id, member.user.id).await;
                    }
                };
                return Err(RoError::Command(CommandError::Blacklist(
                    success.reason.to_owned(),
                )));
            }
        }

        let rankbinds_to_add = guild
            .rankbinds
            .iter()
            .filter(|r| match user_roles.get(&r.group_id) {
                Some(rank_id) => *rank_id == r.rank_id as i64,
                None => r.rank_id == 0,
            })
            .collect::<Vec<_>>();
        let groupbinds_to_add = guild
            .groupbinds
            .iter()
            .filter(|g| user_roles.contains_key(&g.group_id));
        let custombinds_to_add = guild
            .custombinds
            .iter()
            .filter(|c| c.command.evaluate(&command_user).unwrap())
            .collect::<Vec<_>>();
        let mut assetbinds_to_add = Vec::new();
        for asset in &guild.assetbinds {
            if self
                .bot
                .roblox
                .has_asset(user.roblox_id, asset.id, &asset.asset_type.to_string())
                .await?
            {
                assetbinds_to_add.push(asset);
            }
        }

        let roles_to_add = rankbinds_to_add
            .iter()
            .flat_map(|r| r.discord_roles.iter().cloned())
            .chain(groupbinds_to_add.flat_map(|g| g.discord_roles.iter().cloned()))
            .chain(
                custombinds_to_add
                    .iter()
                    .flat_map(|c| c.discord_roles.iter().cloned()),
            )
            .chain(
                assetbinds_to_add
                    .iter()
                    .flat_map(|a| a.discord_roles.iter().cloned()),
            )
            .collect::<Vec<_>>();

        for bind_role in &guild.all_roles {
            let r = RoleId(*bind_role as u64);
            if let Some(_role) = guild_roles.get(&r) {
                if roles_to_add.contains(&bind_role) {
                    if !member.roles.contains(&r) {
                        added_roles.push(r);
                    }
                } else if member.roles.contains(&r) {
                    removed_roles.push(r);
                }
            }
        }

        let nick_bind = rankbinds_to_add
            .iter()
            .sorted_by_key(|r| -r.priority)
            .next();
        let custom = custombinds_to_add
            .iter()
            .sorted_by_key(|c| -c.priority)
            .next();

        let prefix: &str;
        if nick_bind.is_none() && custom.is_none() {
            prefix = "N/A";
        } else if nick_bind.is_none() {
            prefix = &custom.unwrap().prefix;
        } else if custom.is_none() {
            prefix = &nick_bind.unwrap().prefix;
        } else {
            prefix = if custom.unwrap().priority > nick_bind.unwrap().priority {
                &custom.unwrap().prefix
            } else {
                &nick_bind.unwrap().prefix
            };
        }

        let display_name = member
            .nick
            .as_ref()
            .map_or_else(|| Cow::Owned(member.user.name.clone()), Cow::Borrowed);
        let mut disc_nick = display_name.clone();

        let nick_bypass = match server.nickname_bypass {
            Some(n) => member.roles.contains(&n),
            None => false,
        };
        if !nick_bypass {
            if prefix.eq_ignore_ascii_case("N/A") {
                disc_nick = Cow::Borrowed(&username);
            } else if prefix.eq_ignore_ascii_case("Disable") {
            } else {
                disc_nick = Cow::Owned(format!("{} {}", prefix, username));
            }
        }

        if disc_nick.len() > 32 {
            return Err(RoError::Command(CommandError::Miscellanous(format!(
                "The supposed nickname {} was found to be more than 32 characters",
                disc_nick
            ))));
        }

        let update = self.bot.http.update_guild_member(server.id, member.user.id);
        let role_changes = !added_roles.is_empty() || !removed_roles.is_empty();
        let mut roles = member.roles.clone();
        roles.extend_from_slice(&added_roles);
        roles.retain(|r| !removed_roles.contains(r));
        roles = roles.into_iter().unique().collect::<Vec<RoleId>>();

        let nick_changes = disc_nick != display_name;

        if role_changes || nick_changes {
            update
                .roles(roles)
                .nick(disc_nick.to_string())
                .unwrap()
                .await?;
        }

        Ok((added_roles, removed_roles, disc_nick.to_string()))
    }

    pub async fn log_guild(&self, guild_id: GuildId, embed: Embed) {
        let log_channel = self.bot.cache.guild(guild_id).and_then(|g| g.log_channel);
        if let Some(channel_id) = log_channel {
            let _ = self
                .bot
                .http
                .create_message(channel_id)
                .embed(embed)
                .unwrap()
                .await;
        }
    }

    pub async fn log_debug(&self, embed: Embed) {
        let (id, token) = self.bot.webhooks.get("debug").unwrap();
        let _ = self
            .bot
            .http
            .execute_webhook(*id, token)
            .embeds(vec![embed])
            .await;
    }

    pub async fn log_error(&self, text: &str) {
        let (id, token) = self.bot.webhooks.get("error").unwrap();
        let _ = self
            .bot
            .http
            .execute_webhook(*id, token)
            .content(text.to_string())
            .await;
    }

    pub async fn log_premium(&self, text: &str) {
        let (id, token) = self.bot.webhooks.get("premium").unwrap();
        let _ = self
            .bot
            .http
            .execute_webhook(*id, token)
            .content(text.to_string())
            .await;
    }
}
