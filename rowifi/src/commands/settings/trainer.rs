use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::id::RoleId,
    guild::{GuildType, RoGuild},
};

use super::FunctionOption;

#[derive(FromArgs)]
pub struct TrainerArguments {
    #[arg(help = "Option to interact with the custom trainer roles - `add` `remove` `set`")]
    pub option: Option<FunctionOption>,
    #[arg(rest, help = "The discord roles to add, remove or set")]
    pub discord_roles: Option<String>,
}

pub async fn trainer(ctx: CommandContext, args: TrainerArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get() as i64).await?;

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
        FunctionOption::Add => {
            trainer_add(ctx, guild, args.discord_roles.unwrap_or_default()).await
        }
        FunctionOption::Remove => {
            trainer_remove(ctx, guild, args.discord_roles.unwrap_or_default()).await
        }
        FunctionOption::Set => {
            trainer_set(ctx, guild, args.discord_roles.unwrap_or_default()).await
        }
        FunctionOption::View => trainer_view(ctx, guild).await,
    }
}

pub async fn trainer_view(ctx: CommandContext, guild: RoGuild) -> CommandResult {
    let mut description = String::new();
    for trainer_role in guild.trainer_roles {
        description.push_str(&format!("- <@&{}>\n", trainer_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("RoWifi Trainer Roles")
        .description(format!(
            "{}\n\n{}",
            "These are the roles that can interact with trainer commands.", description
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}

pub async fn trainer_add(
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
            roles_to_add.extend(resolved.roles.iter().map(|r| r.id.get() as i64));
        } else if let Some(role_id) = parse_role(role) {
            if server_roles
                .iter()
                .any(|r| r.id == RoleId::new(role_id).unwrap())
            {
                roles_to_add.push(role_id as i64);
            }
        }
    }
    roles_to_add = roles_to_add.into_iter().unique().collect();

    {
        let trainer_roles = ctx.bot.trainer_roles.entry(guild_id).or_default();
        roles_to_add.retain(|r| !trainer_roles.contains(&RoleId::new(*r as u64).unwrap()));
    }

    ctx.bot.database.execute("UPDATE guilds SET trainer_roles = array_cat(trainer_roles, $1::BIGINT[]) WHERE guild_id = $2", &[&roles_to_add, &guild.guild_id]).await?;

    ctx.bot
        .trainer_roles
        .entry(guild_id)
        .or_default()
        .extend(roles_to_add.iter().map(|r| RoleId::new(*r as u64).unwrap()));

    let mut description = "Added Trainer Roles:\n".to_string();
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

pub async fn trainer_remove(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let mut role_ids = Vec::new();
    for r in discord_roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| r.id.get() as i64));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }

    let mut roles_to_keep = guild.admin_roles.clone();
    roles_to_keep.retain(|r| !role_ids.contains(r));
    ctx.bot.database.execute("UPDATE guilds SET trainer_roles = $1 WHERE guild_id = $2", &[&roles_to_keep, &(guild_id.get() as i64)]).await?;

    ctx.bot
        .trainer_roles
        .entry(guild_id)
        .or_default()
        .retain(|r| !role_ids.contains(&(r.0.get() as i64)));

    let mut description = "Removed Trainer Roles:\n".to_string();
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

pub async fn trainer_set(
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
            roles_to_set.extend(resolved.roles.iter().map(|r| r.id.get() as i64));
        } else if let Some(role_id) = parse_role(role) {
            if server_roles
                .iter()
                .any(|r| r.id == RoleId::new(role_id).unwrap())
            {
                roles_to_set.push(role_id as i64);
            }
        }
    }
    roles_to_set = roles_to_set.into_iter().unique().collect::<Vec<_>>();

    ctx.bot.database.execute("UPDATE guilds SET trainer_roles = $1 WHERE guild_id = $2", &[&roles_to_set, &guild.guild_id]).await?;

    ctx.bot.trainer_roles.insert(
        guild_id,
        roles_to_set
            .iter()
            .map(|r| RoleId::new(*r as u64).unwrap())
            .collect(),
    );

    let mut description = "Set Trainer Roles:\n".to_string();
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
