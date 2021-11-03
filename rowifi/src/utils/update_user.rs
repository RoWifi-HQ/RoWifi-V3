use std::{collections::{HashSet, HashMap}, sync::Arc};
use itertools::Itertools;
use rowifi_cache::{CachedGuild, CachedMember};
use rowifi_framework::{context::BotContext, error::RoError};
use rowifi_models::{discord::id::RoleId, guild::{BlacklistActionType, RoGuild}, roblox::id::{UserId as RobloxUserId, AssetId as RobloxAssetId}, rolang::RoCommandUser, user::RoGuildUser, bind::Bind};

pub enum UpdateUserResult {
    Success(Vec<RoleId>, Vec<RoleId>, String),
    Blacklist(String),
    InvalidNickname(String),
    Error(RoError)
}

pub async fn update_user(
    ctx: &BotContext,
    member: Arc<CachedMember>,
    user: &RoGuildUser,
    server: &CachedGuild,
    guild: &RoGuild,
    guild_roles: &HashSet<RoleId>,
    bypass_roblox_cache: bool,
) -> UpdateUserResult {
    let mut added_roles = Vec::<RoleId>::new();
    let mut removed_roles = Vec::<RoleId>::new();

    if let Some(verification_role) = guild.verification_role {
        let verification_role = RoleId::new(verification_role as u64).unwrap();
        if guild_roles.get(&verification_role).is_some()
            && member.roles.contains(&verification_role)
        {
            removed_roles.push(verification_role);
        }
    }

    if let Some(verified_role) = guild.verified_role {
        let verified_role = RoleId::new(verified_role as u64).unwrap();
        if guild_roles.get(&verified_role).is_some() && !member.roles.contains(&verified_role) {
            added_roles.push(verified_role);
        }
    }

    let user_id = RobloxUserId(user.roblox_id as u64);
    let user_roles = match ctx
        .roblox
        .get_user_roles(user_id)
        .await {
            Ok(user_roles) => {
                user_roles.iter()
                .map(|r| (r.group.id.0 as i64, r.role.rank as i64))
                .collect::<HashMap<_, _>>()
            },
            Err(e) => return UpdateUserResult::Error(e.into())
        };

    let roblox_user = match ctx.roblox.get_user(user_id, bypass_roblox_cache).await {
        Ok(r) => r,
        Err(err) => return UpdateUserResult::Error(err.into())
    };
    let command_user = RoCommandUser {
        user,
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
                    let _ = ctx
                        .http
                        .remove_guild_member(server.id, member.user.id)
                        .exec()
                        .await;
                }
                BlacklistActionType::Ban => {
                    let _ = ctx.http.create_ban(server.id, member.user.id).exec().await;
                }
            };
            return UpdateUserResult::Blacklist(success.reason.clone());
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
            roles_to_add.extend(r.discord_roles.iter().copied());
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
            roles_to_add.extend(g.discord_roles.iter().copied());
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
            roles_to_add.extend(c.discord_roles.iter().copied());
        }
    }

    for a in &guild.assetbinds {
        match ctx.roblox.get_asset(user_id, RobloxAssetId(a.id as u64), &a.asset_type.to_string()).await {
            Ok(Some(_)) => {
                if let Some(highest) = nick_bind {
                    if highest.priority() < a.priority() {
                        nick_bind = Some(a);
                    }
                } else {
                    nick_bind = Some(a);
                }
                roles_to_add.extend(a.discord_roles.iter().copied());
            },
            Ok(None) => {},
            Err(err) => return UpdateUserResult::Error(err.into())
        }
    }

    for bind_role in &guild.all_roles {
        let r = RoleId::new(*bind_role as u64).unwrap();
        if guild_roles.get(&r).is_some() {
            if roles_to_add.contains(bind_role) {
                if !member.roles.contains(&r) {
                    added_roles.push(r);
                }
            } else if member.roles.contains(&r) {
                removed_roles.push(r);
            }
        }
    }

    let original_nick = member
        .nick
        .as_ref()
        .map_or_else(|| member.user.name.as_str(), |s| s.as_str());
    let nick_bypass = ctx.has_nickname_bypass(server, &member);
    let nickname = if nick_bypass {
        original_nick.to_string()
    } else {
        nick_bind.map_or_else(
            || roblox_user.name.to_string(),
            |nick_bind| nick_bind.nickname(&roblox_user, user, &member.user.name, &member.nick),
        )
    };

    if nickname.len() > 32 {
        return UpdateUserResult::InvalidNickname(nickname);
    }

    let update = ctx.http.update_guild_member(server.id, member.user.id);
    let role_changes = !added_roles.is_empty() || !removed_roles.is_empty();
    let mut roles = member.roles.clone();
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
            .await {
                return UpdateUserResult::Error(err.into())
            }
    }

    UpdateUserResult::Success(added_roles, removed_roles, nickname)
}