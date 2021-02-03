use framework_new::prelude::*;
use rowifi_models::guild::RoGuild;

use super::BackupArguments;

pub async fn backup_restore(ctx: CommandContext, args: BackupArguments) -> CommandResult {
    match ctx.bot.database.get_premium(ctx.author.id.0).await? {
        Some(p) if p.premium_type.has_backup() => {}
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Backup Failed")
                .unwrap()
                .description("This module may only be used by a Beta Tier user")
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let guild_id = ctx.guild_id.unwrap();
    let name = args.name;
    let existing = ctx.bot.database.get_guild(guild_id.0).await?.is_some();

    let backup = match ctx.bot.database.get_backup(ctx.author.id.0, &name).await? {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Backup Restore Failed")
                .unwrap()
                .description(format!(
                    "No backup with name {} was found associated to your account",
                    name
                ))
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
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

    let guild = RoGuild::from_backup(backup, ctx.bot.http.clone(), guild_id, &roles).await;
    ctx.bot.database.add_guild(guild, existing).await?;
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .content("Backup successfully restored")
        .unwrap()
        .await?;
    Ok(())
}
