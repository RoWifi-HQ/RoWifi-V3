use std::collections::{HashSet, HashMap};
use itertools::Itertools;
use rowifi_cache::{CachedMember, CachedGuild};
use rowifi_framework::{context::BotContext, error::RoError};
use rowifi_models::{user::RoGuildUser, guild::{RoGuild, BlacklistActionType}, bind::Bind, discord::id::RoleId, rolang::RoCommandUser, roblox::id::{UserId as RobloxUserId, AssetId as RobloxAssetId}};

#[allow(dead_code)]
pub struct UpdateUser<'u> {
    pub ctx: &'u BotContext,
    pub member: &'u CachedMember,
    pub user: &'u RoGuildUser,
    pub server: &'u CachedGuild,
    pub guild: &'u RoGuild,
    pub binds: &'u [Bind],
    pub guild_roles: &'u HashSet<RoleId>,
    pub bypass_roblox_cache: bool,
    pub all_roles: &'u [&'u i64]
}

#[allow(dead_code)]
pub enum UpdateUserResult {
    Success(Vec<RoleId>, Vec<RoleId>, String),
    Blacklist(String),
    InvalidNickname(String),
    Error(RoError),
}

impl UpdateUser<'_> {
    #[allow(dead_code)]
    pub async fn execute(self) -> UpdateUserResult {
        let mut added_roles = Vec::<RoleId>::new();
        let mut removed_roles = Vec::<RoleId>::new();

        if let Some(verification_role) = self.guild.verification_role {
            let verification_role = RoleId::new(verification_role as u64).unwrap();
            if self.guild_roles.get(&verification_role).is_some()
                && self.member.roles.contains(&verification_role)
            {
                removed_roles.push(verification_role);
            }
        }

        if let Some(verified_role) = self.guild.verified_role {
            let verified_role = RoleId::new(verified_role as u64).unwrap();
            if self.guild_roles.get(&verified_role).is_some() && !self.member.roles.contains(&verified_role) {
                added_roles.push(verified_role);
            }
        }

        let user_id = RobloxUserId(self.user.roblox_id as u64);
        let user_roles = match self.ctx.roblox.get_user_roles(user_id).await {
            Ok(user_roles) => user_roles
                .iter()
                .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
                .collect::<HashMap<_, _>>(),
            Err(e) => return UpdateUserResult::Error(e.into()),
        };

        let roblox_user = match self.ctx.roblox.get_user(user_id, self.bypass_roblox_cache).await {
            Ok(r) => r,
            Err(err) => return UpdateUserResult::Error(err.into()),
        };
        let command_user = RoCommandUser {
            user: self.user,
            roles: &self.member.roles,
            ranks: &user_roles,
            username: &roblox_user.name,
        };

        if !self.guild.blacklists.is_empty() {
            let success = self.guild
                .blacklists
                .iter()
                .find(|b| b.evaluate(&command_user).unwrap());
            if let Some(success) = success {
                match self.guild.blacklist_action {
                    BlacklistActionType::None => {}
                    BlacklistActionType::Kick => {
                        let _ = self.ctx
                            .http
                            .remove_guild_member(self.server.id, self.member.user.id)
                            .exec()
                            .await;
                    }
                    BlacklistActionType::Ban => {
                        let _ = self.ctx.http.create_ban(self.server.id, self.member.user.id).exec().await;
                    }
                };
                return UpdateUserResult::Blacklist(success.reason.clone());
            }
        }

        let mut nick_bind: Option<&Bind> = None;
        let mut roles_to_add = Vec::new();

        for bind in self.binds {
            match bind {
                Bind::Rank(r) => {
                    let to_add = match user_roles.get(&r.group_id) {
                        Some(rank_id) => *rank_id == r.group_rank_id as i64,
                        None => r.group_rank_id == 0,
                    };
                    if to_add {
                        if let Some(highest) = nick_bind {
                            if highest.priority() < r.priority {
                                nick_bind = Some(bind);
                            }
                        } else {
                            nick_bind = Some(bind);
                        }
                        roles_to_add.extend(r.discord_roles.iter().copied());
                    }
                },
                Bind::Group(g) => {
                    if user_roles.contains_key(&g.group_id) {
                        if let Some(highest) = nick_bind {
                            if highest.priority() < g.priority {
                                nick_bind = Some(bind);
                            }
                        } else {
                            nick_bind = Some(bind);
                        }
                        roles_to_add.extend(g.discord_roles.iter().copied());
                    }
                },
                Bind::Custom(c) => {
                    if c.command.evaluate(&command_user).unwrap() {
                        if let Some(highest) = nick_bind {
                            if highest.priority() < c.priority {
                                nick_bind = Some(bind);
                            }
                        } else {
                            nick_bind = Some(bind);
                        }
                        roles_to_add.extend(c.discord_roles.iter().copied());
                    }
                },
                Bind::Asset(a) => {
                    match self.ctx
                        .roblox
                        .get_asset(
                            user_id,
                            RobloxAssetId(a.asset_id as u64),
                            &a.asset_type.to_string(),
                        )
                        .await
                    {
                        Ok(Some(_)) => {
                            if let Some(highest) = nick_bind {
                                if highest.priority() < a.priority {
                                    nick_bind = Some(bind);
                                }
                            } else {
                                nick_bind = Some(bind);
                            }
                            roles_to_add.extend(a.discord_roles.iter().copied());
                        }
                        Ok(None) => {}
                        Err(err) => return UpdateUserResult::Error(err.into()),
                    }
                }
            }
        }

        for bind_role in self.all_roles {
            let r = RoleId::new(**bind_role as u64).unwrap();
            if self.guild_roles.get(&r).is_some() {
                if roles_to_add.contains(bind_role) {
                    if !self.member.roles.contains(&r) {
                        added_roles.push(r);
                    }
                } else if self.member.roles.contains(&r) {
                    removed_roles.push(r);
                }
            }
        }

        let original_nick = self.member
            .nick
            .as_ref()
            .map_or_else(|| self.member.user.name.as_str(), String::as_str);
        let nick_bypass = self.ctx.has_nickname_bypass(self.server, &self.member);
        let nickname = if nick_bypass {
            original_nick.to_string()
        } else {
            nick_bind.map_or_else(
                || roblox_user.name.to_string(),
                |nick_bind| nick_bind.nickname(&roblox_user, self.user, &self.member.user.name),
            )
        };

        if nickname.len() > 32 {
            return UpdateUserResult::InvalidNickname(nickname);
        }

        let update = self.ctx.http.update_guild_member(self.server.id, self.member.user.id);
        let role_changes = !added_roles.is_empty() || !removed_roles.is_empty();
        let mut roles = self.member.roles.clone();
        roles.extend_from_slice(&added_roles);
        roles.retain(|r| !removed_roles.contains(r));
        roles = roles.into_iter().unique().collect::<Vec<RoleId>>();

        let nick_changes = nickname != original_nick;

        if role_changes || nick_changes {
            if let Err(err) = update
                .roles(&roles)
                .nick(Some(&nickname))
                .unwrap()
                .exec()
                .await
            {
                return UpdateUserResult::Error(err.into());
            }
        }

        UpdateUserResult::Success(added_roles, removed_roles, nickname)
    }
}