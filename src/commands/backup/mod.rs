mod new;
mod restore;

use crate::framework::prelude::*;

pub use new::*;
pub use restore::*;

pub static BACKUP_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["backup"],
    desc: Some("Command to view saved backups"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[&BACKUP_NEW_COMMAND, &BACKUP_RESTORE_COMMAND],
    group: Some("Premium"),
};

pub static BACKUP_COMMAND: Command = Command {
    fun: backup,
    options: &BACKUP_OPTIONS,
};

#[command]
pub async fn backup(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
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

    let backups = ctx.database.get_backups(msg.author.id.0).await?;
    let mut embed = EmbedBuilder::new().default_data().title("Backups").unwrap();

    for backup in backups {
        let val = format!("Prefix: {}\nVerification: {}\nVerified: {}\nRankbinds: {}\nGroupbinds: {}\nCustombinds: {}\nAssetbinds: {}",
            backup.command_prefix.unwrap_or_else(|| "!".into()), backup.verification_role.unwrap_or_default(), backup.verified_role.unwrap_or_default(),
            backup.rankbinds.len(), backup.groupbinds.len(), backup.custombinds.len(), backup.assetbinds.len()
        );
        embed = embed.field(EmbedFieldBuilder::new(backup.name, val).unwrap());
    }

    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed.build().unwrap())
        .unwrap()
        .await?;
    Ok(())
}
