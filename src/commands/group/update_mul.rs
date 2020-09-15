use crate::framework::prelude::*;
use crate::models::guild::GuildType;
use twilight_model::id::UserId;

pub static UPDATE_ALL_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update-all"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: Some("Premium")
};

pub static UPDATE_ALL_COMMAND: Command = Command {
    fun: update_all,
    options: &UPDATE_ALL_OPTIONS
};

pub static UPDATE_ROLE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update-role"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
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
        return Ok(())
    }

    let role_id = match args.next().map(parse_role) {
        Some(Some(r)) => r,
        Some(None) => return Ok(()),
        None => return Ok(())
    };
    let role_id = RoleId(role_id);

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