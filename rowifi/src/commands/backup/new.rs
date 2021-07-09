use rowifi_framework::prelude::*;
use std::collections::HashMap;

use super::BackupArguments;

pub async fn backup_new(ctx: CommandContext, args: BackupArguments) -> CommandResult {
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
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let name = args.name;
    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles = HashMap::new();
    for role in server_roles {
        let cached = ctx.bot.cache.role(role);
        if let Some(cached) = cached {
            roles.insert(role, cached.name.clone());
        }
    }

    let backup = guild.to_backup(ctx.author.id.0 as i64, &name, &roles);
    ctx.bot.database.add_backup(backup, &name).await?;
    ctx.respond()
        .content(format!("New backup with {} was created", name))
        .await?;
    Ok(())
}
