use crate::framework::prelude::*;
use crate::models::guild::GuildType;
use twilight_model::id::UserId;

pub static UPDATE_ALL_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["update-all"],
    desc: Some("Command to update all members in the server"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Premium")
};

pub static UPDATE_ALL_COMMAND: Command = Command {
    fun: update_all,
    options: &UPDATE_ALL_OPTIONS
};

pub static UPDATE_ROLE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["update-role"],
    desc: Some("Command to update all members with a certain role"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: Some("Premium")
};

pub static UPDATE_ROLE_COMMAND: Command = Command {
    fun: update_role,
    options: &UPDATE_ROLE_OPTIONS
};

#[command]
pub async fn update_all(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Update All Failed").unwrap()
            .description("This command may only be used in Premium Servers").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
        return Ok(())
    }
    let server = ctx.cache.guild(guild_id).unwrap();
    let members = ctx.cache.members(guild_id).into_iter().map(|m| m.0).collect::<Vec<_>>();
    let users = ctx.database.get_users(members).await?;
    let guild_roles = ctx.cache.roles(guild_id);
    let c = ctx.clone();
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {continue;}
                }
                tracing::trace!(id = user.discord_id, "Mass Update for member");
                let _ = user.update(c.http.clone(), member, c.roblox.clone(), server.clone(), &guild, &guild_roles).await;
            }
        }
    });
    Ok(())
}

#[command]
pub async fn update_role(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Update All Failed").unwrap()
            .description("This command may only be used in Premium Servers").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
        return Ok(())
    }

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let role_str = match args.next() {
        Some(r) => r,
        None => return Ok(())
    };
    let role_id = match parse_role(role_str) {
        Some(v) if server_roles.contains(&RoleId(v)) => RoleId(v),
        _ => return Err(CommandError::ParseArgument(role_str.into(), "Role".into(), "Discord Role/Number".into()).into())
    };

    let server = ctx.cache.guild(guild_id).unwrap();
    let members = ctx.cache.members(guild_id).into_iter().map(|m| m.0).collect::<Vec<_>>();
    let users = ctx.database.get_users(members).await?;
    let guild_roles = ctx.cache.roles(guild_id);
    let c = ctx.clone();
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if !member.roles.contains(&role_id) {continue;}
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {continue;}
                }
                tracing::trace!(id = user.discord_id, "Mass Update for member");
                let _ = user.update(c.http.clone(), member, c.roblox.clone(), server.clone(), &guild, &guild_roles).await;
            }
        }
    });
    Ok(())
}