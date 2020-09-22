use crate::framework::prelude::*;
use std::collections::HashMap;

pub static BACKUP_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to create a new backup"),
    usage: Some("backup new <Name>"),
    examples: &["backup new RoWifi"],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static BACKUP_NEW_COMMAND: Command = Command {
    fun: backup_new,
    options: &BACKUP_NEW_OPTIONS
};

#[command]
pub async fn backup_new(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let name = match args.next() {
        Some(g) => g.to_owned(),
        None => return Ok(())
    };
    
    let server_roles = ctx.cache.roles(guild_id);
    let mut roles = HashMap::new();
    for role in server_roles {
        let cached = ctx.cache.role(role);
        if let Some(cached) = cached {
            roles.insert(role, cached);
        }
    }

    let backup = guild.to_backup(msg.author.id.0 as i64, &name, &roles);
    ctx.database.add_backup(backup, &name).await?;
    let _ = ctx.http.create_message(msg.channel_id).content(format!("New backup with {} was created", name)).unwrap().await?;
    Ok(())
}