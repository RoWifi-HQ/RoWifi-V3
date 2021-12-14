use rowifi_database::postgres::types::Json;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{AssetbindBackup, Bind, BindBackup, CustombindBackup, GroupbindBackup, RankbindBackup},
    discord::id::RoleId,
    guild::backup::{GuildBackup, GuildBackupData},
    user::{RoUser, UserFlags},
};
use std::collections::HashMap;

use super::BackupArguments;

pub async fn backup_new(ctx: CommandContext, args: BackupArguments) -> CommandResult {
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
    let guild = ctx.bot.database.get_guild(guild_id.0.get() as i64).await?;
    let binds = ctx
        .bot
        .database
        .query::<Bind>(
            "SELECT * FROM binds WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    let name = args.name;
    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles = HashMap::new();
    for role in server_roles {
        let cached = ctx.bot.cache.role(role);
        if let Some(cached) = cached {
            roles.insert(role, cached.name.clone());
        }
    }

    let server_channels = ctx.bot.cache.guild_channels(guild_id);
    let mut channels = HashMap::new();
    for channel in server_channels {
        let cached = ctx.bot.cache.channel(channel);
        if let Some(cached) = cached {
            channels.insert(channel, cached.name().to_string());
        }
    }

    let verification_roles = guild
        .verification_roles
        .iter()
        .filter_map(|r| roles.get(&RoleId::new(*r as u64).unwrap()).cloned())
        .collect();
    let verified_roles = guild
        .verified_roles
        .iter()
        .filter_map(|r| roles.get(&RoleId::new(*r as u64).unwrap()).cloned())
        .collect();
    let binds = binds
        .into_iter()
        .map(|b| {
            let discord_roles = b
                .discord_roles()
                .iter()
                .filter_map(|r| roles.get(&RoleId::new(*r as u64).unwrap()).cloned())
                .collect();

            match b {
                Bind::Rank(r) => BindBackup::Rank(RankbindBackup {
                    group_id: r.group_id,
                    group_rank_id: r.group_rank_id,
                    roblox_rank_id: r.roblox_rank_id,
                    discord_roles,
                    template: r.template,
                    priority: r.priority,
                }),
                Bind::Group(g) => BindBackup::Group(GroupbindBackup {
                    group_id: g.group_id,
                    discord_roles,
                    template: g.template,
                    priority: g.priority,
                }),
                Bind::Custom(c) => BindBackup::Custom(CustombindBackup {
                    custom_bind_id: c.custom_bind_id,
                    code: c.code,
                    discord_roles,
                    template: c.template,
                    priority: c.priority,
                }),
                Bind::Asset(a) => BindBackup::Asset(AssetbindBackup {
                    asset_id: a.asset_id,
                    asset_type: a.asset_type,
                    discord_roles,
                    template: a.template,
                    priority: a.priority,
                }),
            }
        })
        .collect();

    let backup = GuildBackup {
        backup_id: 0,
        discord_id: ctx.author.id.get() as i64,
        name: name.clone(),
        data: Json(GuildBackupData {
            command_prefix: guild.command_prefix,
            verification_roles,
            verified_roles,
            blacklists: guild.blacklists,
            blacklist_action: guild.blacklist_action,
            update_on_join: guild.update_on_join,
            binds,
        }),
    };

    ctx.bot.database.execute("INSERT INTO backups(discord_id, name, data) VALUES($1, $2, $3) ON CONFLICT DO UPDATE SET data = $3", &[&backup.discord_id, &backup.name, &backup.data]).await?;

    ctx.respond()
        .content(&format!("New backup with {} was created", name))?
        .exec()
        .await?;
    Ok(())
}
