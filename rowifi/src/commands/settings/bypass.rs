use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::{GuildType, RoGuild};
use twilight_model::id::RoleId;

use super::FunctionOption;

#[derive(FromArgs)]
pub struct BypassArguments {
    #[arg(help = "Option to interact with the custom bypass roles")]
    pub option: Option<FunctionOption>,
    #[arg(rest, "The discord roles to add, remove or set")]
    pub discord_roles: Option<String>,
}

pub async fn bypass(ctx: CommandContext, args: BypassArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    if let Some(option) = args.option {
        match option {
            FunctionOption::Add => {
                bypass_add(ctx, guild, args.discord_roles.unwrap_or_default()).await
            }
            FunctionOption::Remove => {
                bypass_remove(ctx, guild, args.discord_roles.unwrap_or_default()).await
            }
            FunctionOption::Set => {
                bypass_set(ctx, guild, args.discord_roles.unwrap_or_default()).await
            }
        }
    } else {
        bypass_view(ctx, guild).await
    }
}

pub async fn bypass_view(ctx: CommandContext, guild: RoGuild) -> CommandResult {
    let mut description = String::new();
    for bypass_role in guild.settings.bypass_roles {
        description.push_str(&format!("- <@&{}>\n", bypass_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bypass Roles")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}

pub async fn bypass_add(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = discord_roles.split_ascii_whitespace().collect::<Vec<_>>();
    let mut roles_to_add = Vec::new();
    for role in roles {
        if let Some(role_id) = parse_role(role) {
            if server_roles.iter().any(|r| r.id == RoleId(role_id)) {
                roles_to_add.push(role_id as i64);
            }
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Settings.BypassRoles": {"$each": roles_to_add.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    ctx.bot
        .bypass_roles
        .entry(guild_id)
        .or_default()
        .extend(roles_to_add.iter().map(|r| RoleId(*r as u64)));

    let mut description = "Added Bypass Roles:\n".to_string();
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
    ctx.respond().embed(embed).await?;

    Ok(())
}

pub async fn bypass_remove(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

    let mut role_ids = Vec::new();
    for r in discord_roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pullAll": {"Settings.BypassRoles": role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;

    ctx.bot
        .bypass_roles
        .entry(guild_id)
        .or_default()
        .retain(|r| role_ids.contains(&r.0));

    let mut description = "Removed Bypass Roles:\n".to_string();
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
    ctx.respond().embed(embed).await?;

    Ok(())
}

pub async fn bypass_set(
    ctx: CommandContext,
    guild: RoGuild,
    discord_roles: String,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = discord_roles.split_ascii_whitespace().collect::<Vec<_>>();
    let mut roles_to_set = Vec::new();
    for role in roles {
        if let Some(role_id) = parse_role(role) {
            if server_roles.iter().any(|r| r.id == RoleId(role_id)) {
                roles_to_set.push(role_id as i64);
            }
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"Settings.BypassRoles": &roles_to_set}};
    ctx.bot.database.modify_guild(filter, update).await?;

    ctx.bot.admin_roles.insert(
        guild_id,
        roles_to_set.iter().map(|r| RoleId(*r as u64)).collect(),
    );

    let mut description = "Set Bypass Roles:\n".to_string();
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
    ctx.respond().embed(embed).await?;

    Ok(())
}
