use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::{GuildType, RoGuild},
    id::RoleId,
};

use super::FunctionOption;

#[derive(FromArgs)]
pub struct AdminArguments {
    #[arg(help = "Option to interact with the custom admin roles- `add` `remove` `set`")]
    pub option: Option<FunctionOption>,
    #[arg(rest, help = "The discord roles to add, remove or set")]
    pub discord_roles: Option<String>,
}

pub async fn admin(ctx: CommandContext, args: AdminArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    if guild.kind == GuildType::Free {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let option = args.option.unwrap_or_default();
    match option {
        FunctionOption::Add => admin_add(ctx, guild, args.discord_roles.unwrap_or_default()).await,
        FunctionOption::Remove => {
            admin_remove(ctx, guild, args.discord_roles.unwrap_or_default()).await
        }
        FunctionOption::Set => admin_set(ctx, guild, args.discord_roles.unwrap_or_default()).await,
        FunctionOption::View => admin_view(ctx, guild).await,
    }
}

pub async fn admin_view(ctx: CommandContext, guild: RoGuild) -> CommandResult {
    let mut description = String::new();
    for admin_role in guild.admin_roles {
        description.push_str(&format!("- <@&{}>\n", admin_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("RoWifi Admin Roles")
        .description(format!("{}\n\n{}", "These are the roles that can manage RoWifi. They are **not** roles with administrator permissions on discord. However, by default, anyone with administrator permissions can manage RoWifi.", description))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}

pub async fn admin_add(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = discord_roles.split_ascii_whitespace().collect::<Vec<_>>();
    let mut roles_to_add = Vec::new();
    for role in roles {
        if let Some(resolved) = &ctx.resolved {
            roles_to_add.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(role_id) = parse_role(role) {
            if server_roles.iter().any(|r| r.id == role_id) {
                roles_to_add.push(role_id);
            }
        }
    }
    roles_to_add = roles_to_add.into_iter().unique().collect();

    {
        let admin_roles = ctx.bot.admin_roles.entry(guild_id).or_default();
        roles_to_add.retain(|r| !admin_roles.contains(r));
    }

    ctx.bot.database.execute("UPDATE guilds SET admin_roles = array_cat(admin_roles, $1::BIGINT[]) WHERE guild_id = $2", &[&roles_to_add, &guild.guild_id]).await?;

    ctx.bot
        .admin_roles
        .entry(guild_id)
        .or_default()
        .extend(roles_to_add.iter().copied());

    let mut description = "Added Admin Roles:\n".to_string();
    for role in roles_to_add {
        description.push_str(&format!("- <@&{}>\n", role));
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}

pub async fn admin_remove(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let mut role_ids = Vec::new();
    for r in discord_roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }

    let mut roles_to_keep = guild.admin_roles.clone();
    roles_to_keep.retain(|r| !role_ids.contains(r));
    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET admin_roles = $1 WHERE guild_id = $2",
            &[&roles_to_keep, &(guild_id)],
        )
        .await?;

    ctx.bot
        .admin_roles
        .entry(guild_id)
        .or_default()
        .retain(|r| !role_ids.contains(r));

    let mut description = "Removed Admin Roles:\n".to_string();
    for role in role_ids {
        description.push_str(&format!("- <@&{}>\n", role));
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}

pub async fn admin_set(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = discord_roles.split_ascii_whitespace().collect::<Vec<_>>();
    let mut roles_to_set = Vec::new();
    for role in roles {
        if let Some(resolved) = &ctx.resolved {
            roles_to_set.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(role_id) = parse_role(role) {
            if server_roles.iter().any(|r| r.id == role_id) {
                roles_to_set.push(role_id);
            }
        }
    }
    roles_to_set = roles_to_set.into_iter().unique().collect::<Vec<_>>();

    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET admin_roles = $1 WHERE guild_id = $2",
            &[&roles_to_set, &guild.guild_id],
        )
        .await?;

    ctx.bot.admin_roles.insert(guild_id, roles_to_set.clone());

    let mut description = "Set Admin Roles:\n".to_string();
    for role in roles_to_set {
        description.push_str(&format!("- <@&{}>\n", role));
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
