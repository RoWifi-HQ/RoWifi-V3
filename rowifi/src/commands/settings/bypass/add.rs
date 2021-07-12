use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct BypassAddArguments {
    #[arg(rest, help = "List of all roles to be added as `RoWifi Bypass`")]
    pub roles: String,
}

pub async fn bypass_add(ctx: CommandContext, args: BypassAddArguments) -> CommandResult {
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

    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = args.roles.split_ascii_whitespace().collect::<Vec<_>>();
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

#[derive(FromArgs)]
pub struct NicknameBypassAddArguments {
    #[arg(
        rest,
        help = "List of all roles to be added as `RoWifi Nickname Bypass`"
    )]
    pub roles: String,
}

pub async fn nickname_bypass_add(
    ctx: CommandContext,
    args: NicknameBypassAddArguments,
) -> CommandResult {
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

    let server_roles = ctx.bot.cache.guild_roles(guild_id);
    let roles = args.roles.split_ascii_whitespace().collect::<Vec<_>>();
    let mut roles_to_add = Vec::new();
    for role in roles {
        if let Some(role_id) = parse_role(role) {
            if server_roles.iter().any(|r| r.id == RoleId(role_id)) {
                roles_to_add.push(role_id as i64);
            }
        }
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Settings.NicknameBypassRoles": {"$each": roles_to_add.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    ctx.bot
        .nickname_bypass_roles
        .entry(guild_id)
        .or_default()
        .extend(roles_to_add.iter().map(|r| RoleId(*r as u64)));

    let mut description = "Added Nickname Bypass Roles:\n".to_string();
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
