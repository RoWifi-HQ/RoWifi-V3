use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::id::{ChannelId, RoleId},
    guild::{GuildType, RoGuild},
};
use std::collections::HashMap;

use super::BackupArguments;

pub async fn backup_restore(ctx: CommandContext, args: BackupArguments) -> CommandResult {
    match ctx.bot.database.get_premium(ctx.author.id.0).await? {
        Some(p) if p.premium_type.has_backup() => {}
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Backup Failed")
                .description("This module may only be used by a Beta Tier user")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };

    let guild_id = ctx.guild_id.unwrap();
    let name = args.name;
    ctx.bot.database.get_guild(guild_id.0).await?;

    let backup = match ctx.bot.database.get_backup(ctx.author.id.0, &name).await? {
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
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles = Vec::new();
    for role in server_roles {
        let cached = ctx.bot.cache.role(role);
        if let Some(cached) = cached {
            roles.push((cached.id, cached.name.clone()));
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

    let guild =
        RoGuild::from_backup(backup, ctx.bot.http.clone(), guild_id, &roles, &channels).await;
    ctx.bot.database.add_guild(&guild, true).await?;

    if guild.settings.guild_type != GuildType::Normal {
        ctx.bot.admin_roles.insert(
            guild_id,
            guild
                .settings
                .admin_roles
                .iter()
                .map(|r| RoleId(*r as u64))
                .collect(),
        );
        ctx.bot.trainer_roles.insert(
            guild_id,
            guild
                .settings
                .trainer_roles
                .iter()
                .map(|r| RoleId(*r as u64))
                .collect(),
        );
        ctx.bot.bypass_roles.insert(
            guild_id,
            guild
                .settings
                .bypass_roles
                .iter()
                .map(|r| RoleId(*r as u64))
                .collect(),
        );
        ctx.bot.nickname_bypass_roles.insert(
            guild_id,
            guild
                .settings
                .nickname_bypass_roles
                .iter()
                .map(|r| RoleId(*r as u64))
                .collect(),
        );
    }
    if let Some(log_channel) = guild.settings.log_channel {
        ctx.bot
            .log_channels
            .insert(guild_id, ChannelId(log_channel as u64));
    }
    if let Some(prefix) = guild.command_prefix {
        ctx.bot.prefixes.insert(guild_id, prefix);
    }

    ctx.respond()
        .content("Backup successfully restored")
        .exec()
        .await?;
    Ok(())
}
