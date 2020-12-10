use itertools::Itertools;
use patreon::Client as PatreonClient;
use roblox::Client as RobloxClient;
use rowifi_cache::{Cache, CachedGuild, CachedMember};
use rowifi_database::Database;
use rowifi_models::{
    guild::{BlacklistActionType, RoGuild},
    rolang::RoCommandUser,
    stats::BotStats,
    user::RoUser,
};
use std::{borrow::Cow, collections::HashSet, sync::Arc};
use twilight_gateway::Cluster;
use twilight_http::Client as Http;
use twilight_model::id::{GuildId, RoleId, UserId};
use twilight_standby::Standby;

use super::{logger::Logger, BotConfig, CommandError, Configuration, RoError};

#[derive(Clone)]
pub struct Context {
    pub http: Http,
    pub cache: Cache,
    pub database: Database,
    pub roblox: RobloxClient,
    pub standby: Standby,
    pub cluster: Cluster,
    pub logger: Arc<Logger>,
    pub config: Arc<Configuration>,
    pub patreon: PatreonClient,
    pub stats: Arc<BotStats>,
    pub bot_config: Arc<BotConfig>,
}

impl Context {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        http: Http,
        cache: Cache,
        database: Database,
        roblox: RobloxClient,
        standby: Standby,
        cluster: Cluster,
        logger: Arc<Logger>,
        config: Arc<Configuration>,
        patreon: PatreonClient,
        stats: Arc<BotStats>,
        bot_config: Arc<BotConfig>,
    ) -> Self {
        Self {
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
            bot_config,
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

    pub async fn update_user(
        &self,
        member: Arc<CachedMember>,
        user: &RoUser,
        server: Arc<CachedGuild>,
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

        let user_roles = self.roblox.get_user_roles(user.roblox_id).await?;
        let username = self.roblox.get_username_from_id(user.roblox_id).await?;
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
                            .http
                            .remove_guild_member(server.id, member.user.id)
                            .await;
                    }
                    BlacklistActionType::Ban => {
                        let _ = self.http.create_ban(server.id, member.user.id).await;
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
            .collect_vec();
        let groupbinds_to_add = guild
            .groupbinds
            .iter()
            .filter(|g| user_roles.contains_key(&g.group_id));
        let custombinds_to_add = guild
            .custombinds
            .iter()
            .filter(|c| c.command.evaluate(&command_user).unwrap())
            .collect_vec();
        let mut assetbinds_to_add = Vec::new();
        for asset in &guild.assetbinds {
            if self
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
            .collect_vec();

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
            return Err(RoError::Command(CommandError::NicknameTooLong(format!(
                "The supposed nickname {} was found to be more than 32 characters",
                disc_nick
            ))));
        }

        let update = self.http.update_guild_member(server.id, member.user.id);
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
}
