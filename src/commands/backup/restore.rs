use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static BACKUP_RESTORE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["restore"],
    desc: Some("Command to restore a backup"),
    usage: Some("backup restore <Name>"),
    examples: &["backup restore RoWifi"],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static BACKUP_RESTORE_COMMAND: Command = Command {
    fun: backup_restore,
    options: &BACKUP_RESTORE_OPTIONS,
};

#[command]
pub async fn backup_restore(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    match ctx.database.get_premium(msg.author.id.0).await? {
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let guild_id = msg.guild_id.unwrap();
    let name = match args.next() {
        Some(g) => g.to_owned(),
        None => return Ok(()),
    };
    let existing = ctx.database.get_guild(guild_id.0).await?.is_some();

    let backup = match ctx.database.get_backup(msg.author.id.0, &name).await? {
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let server_roles = ctx.cache.roles(guild_id);
    let mut roles = Vec::new();
    for role in server_roles {
        let cached = ctx.cache.role(role);
        if let Some(cached) = cached {
            roles.push(cached);
        }
    }

    let guild = RoGuild::from_backup(backup, ctx, guild_id, &roles).await;
    ctx.database.add_guild(guild, existing).await?;
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .content("Backup successfully restored")
        .unwrap()
        .await?;
    Ok(())
}
