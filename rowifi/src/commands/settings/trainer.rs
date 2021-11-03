use mongodb::bson::doc;
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
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    if guild.settings.guild_type == GuildType::Normal {
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
    for trainer_role in guild.settings.trainer_roles {
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
        if let Some(role_id) = parse_role(role) {
            if server_roles
                .iter()
                .any(|r| r.id == RoleId::new(role_id).unwrap())
                && !ctx
                    .bot
                    .trainer_roles
                    .entry(guild_id)
                    .or_default()
                    .contains(&RoleId::new(role_id).unwrap())
            {
                roles_to_add.push(role_id as i64);
            }
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Settings.TrainerRoles": {"$each": roles_to_add.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pullAll": {"Settings.TrainerRoles": role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
        if let Some(role_id) = parse_role(role) {
            if server_roles
                .iter()
                .any(|r| r.id == RoleId::new(role_id).unwrap())
            {
                roles_to_set.push(role_id as i64);
            }
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"Settings.TrainerRoles": &roles_to_set}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
