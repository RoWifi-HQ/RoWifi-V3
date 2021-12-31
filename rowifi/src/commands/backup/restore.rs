use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{Assetbind, Bind, BindBackup, BindType, Custombind, Groupbind, Rankbind},
    guild::{backup::GuildBackup, GuildType, RoGuild},
    rolang::RoCommand,
    user::{RoUser, UserFlags},
    id::RoleId,
};
use std::collections::HashMap;

use super::BackupArguments;

pub async fn backup_restore(ctx: CommandContext, args: BackupArguments) -> CommandResult {
    match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(ctx.author.id.get() as i64)],
        )
        .await?
    {
        Some(u) if u.flags.contains(UserFlags::BETA) => {}
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Backup Failed")
                .description("This module may only be used by a Beta Tier user")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let guild_id = ctx.guild_id.unwrap();
    let name = args.name;
    ctx.bot.database.get_guild(guild_id).await?;

    let backup = match ctx
        .bot
        .database
        .query_opt::<GuildBackup>(
            "SELECT * FROM backup WHERE discord_id = $1 AND name = $2",
            &[&(ctx.author.id.get() as i64), &name],
        )
        .await?
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Backup Restore Failed")
                .description(format!(
                    "No backup with name {} was found associated to your account",
                    name
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let all_roles = backup
        .data
        .0
        .binds
        .iter()
        .flat_map(|b| b.discord_roles())
        .cloned()
        .unique()
        .collect::<Vec<_>>();

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut existing_roles = HashMap::new();
    for role in server_roles {
        let cached = ctx.bot.cache.role(role);
        if let Some(cached) = cached {
            existing_roles.insert(cached.name.clone(), cached.id);
        }
    }

    let mut roles_map = HashMap::new();
    for r in all_roles {
        if let Some(existing) = existing_roles.get(&r) {
            roles_map.insert(r, *existing);
        } else {
            let role = ctx
                .bot
                .http
                .create_role(guild_id.0)
                .name(&r)
                .exec()
                .await?
                .model()
                .await?;
            roles_map.insert(role.name, RoleId(role.id));
        }
    }

    let server_channels = ctx.bot.cache.guild_channels(guild_id);
    let mut channels = HashMap::new();
    for channel in server_channels {
        let cached = ctx.bot.cache.channel(channel);
        if let Some(cached) = cached {
            channels.insert(cached.name().to_string(), channel);
        }
    }

    let data = backup.data.0;

    let verification_roles = data
        .verification_roles
        .iter()
        .filter_map(|v| roles_map.get(v).cloned())
        .collect();
    let verified_roles = data
        .verified_roles
        .iter()
        .filter_map(|v| roles_map.get(v).cloned())
        .collect();

    let binds = data
        .binds
        .into_iter()
        .map(|b| {
            let discord_roles = b
                .discord_roles()
                .iter()
                .filter_map(|v| roles_map.get(v).cloned())
                .collect();

            match b {
                BindBackup::Rank(r) => Bind::Rank(Rankbind {
                    bind_id: 0,
                    group_id: r.group_id,
                    group_rank_id: r.group_rank_id,
                    roblox_rank_id: r.roblox_rank_id,
                    discord_roles,
                    template: r.template,
                    priority: r.priority,
                }),
                BindBackup::Group(g) => Bind::Group(Groupbind {
                    bind_id: 0,
                    group_id: g.group_id,
                    discord_roles,
                    template: g.template,
                    priority: g.priority,
                }),
                BindBackup::Custom(c) => Bind::Custom(Custombind {
                    bind_id: 0,
                    custom_bind_id: c.custom_bind_id,
                    code: c.code.clone(),
                    command: RoCommand::new(&c.code).unwrap(),
                    discord_roles,
                    template: c.template,
                    priority: c.priority,
                }),
                BindBackup::Asset(a) => Bind::Asset(Assetbind {
                    bind_id: 0,
                    asset_id: a.asset_id,
                    asset_type: a.asset_type,
                    discord_roles,
                    template: a.template,
                    priority: a.priority,
                }),
            }
        })
        .collect::<Vec<_>>();

    let guild = RoGuild {
        guild_id: ctx.guild_id.unwrap(),
        command_prefix: data.command_prefix,
        verification_roles,
        verified_roles,
        blacklists: data.blacklists,
        disabled_channels: Vec::new(),
        registered_groups: Vec::new(),
        auto_detection: false,
        kind: GuildType::Free,
        premium_owner: None,
        blacklist_action: data.blacklist_action,
        update_on_join: data.update_on_join,
        admin_roles: Vec::new(),
        trainer_roles: Vec::new(),
        bypass_roles: Vec::new(),
        nickname_bypass_roles: Vec::new(),
        log_channel: None,
    };

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let delete_guild = transaction
        .prepare_cached("DELETE FROM guilds WHERE guild_id = $1")
        .await?;
    transaction
        .execute(&delete_guild, &[&guild.guild_id])
        .await?;
    let insert_guild = transaction.prepare_cached("INSERT INTO guilds(guild_id, kind, command_prefix, verification_roles, verified_roles, blacklists, blacklist_action, update_on_join) VALUES($1, $2, $3, $4, $5, $6, $7, $8)").await?;
    transaction
        .execute(&insert_guild, &[&guild.guild_id])
        .await?;

    let delete_binds = transaction
        .prepare_cached("DELETE FROM binds WHERE guild_id = $1")
        .await?;
    transaction
        .execute(&delete_binds, &[&guild.guild_id])
        .await?;

    let add_rank = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, group_id, group_rank_id, roblox_rank_id, template, priority, discord_roles) VALUES($1, $2, $3, $4, $5, $6, $7, $8)").await?;
    let add_group = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, group_id, discord_roles, priority, template) VALUES($1, $2, $3, $4, $5, $6)").await?;
    let add_custom = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, custom_bind_id, discord_roles, code, priority, template) VALUES($1, $2, $3, $4, $5, $6, $7)").await?;
    let add_asset = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, asset_id, asset_type, discord_roles, priority, template) VALUES($1, $2, $3, $4, $5, $6, $7)").await?;
    for bind in binds {
        match bind {
            Bind::Rank(r) => {
                transaction
                    .execute(
                        &add_rank,
                        &[
                            &BindType::Rank,
                            &guild.guild_id,
                            &r.group_id,
                            &r.group_rank_id,
                            &r.roblox_rank_id,
                            &r.template,
                            &r.priority,
                            &r.discord_roles,
                        ],
                    )
                    .await?
            }
            Bind::Group(g) => {
                transaction
                    .execute(
                        &add_group,
                        &[
                            &BindType::Group,
                            &guild.guild_id,
                            &g.group_id,
                            &g.discord_roles,
                            &g.priority,
                            &g.template,
                        ],
                    )
                    .await?
            }
            Bind::Custom(c) => {
                transaction
                    .execute(
                        &add_custom,
                        &[
                            &BindType::Custom,
                            &guild.guild_id,
                            &c.custom_bind_id,
                            &c.discord_roles,
                            &c.code,
                            &c.priority,
                            &c.template,
                        ],
                    )
                    .await?
            }
            Bind::Asset(a) => {
                transaction
                    .execute(
                        &add_asset,
                        &[
                            &BindType::Asset,
                            &guild.guild_id,
                            &a.asset_id,
                            &a.asset_type,
                            &a.discord_roles,
                            &a.priority,
                            &a.template,
                        ],
                    )
                    .await?
            }
        };
    }
    transaction.commit().await?;

    ctx.bot.admin_roles.insert(guild_id, Vec::new());
    ctx.bot.trainer_roles.insert(guild_id, Vec::new());
    ctx.bot.bypass_roles.insert(guild_id, Vec::new());
    ctx.bot.nickname_bypass_roles.insert(guild_id, Vec::new());

    ctx.bot.log_channels.remove(&guild_id);
    ctx.bot.prefixes.insert(guild_id, guild.command_prefix);

    ctx.respond()
        .content("Backup successfully restored")?
        .exec()
        .await?;
    Ok(())
}
