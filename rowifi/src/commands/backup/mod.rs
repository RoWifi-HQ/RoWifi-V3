mod new;
mod restore;

use framework_new::prelude::*;

pub use new::*;
pub use restore::*;

pub fn backup_config(cmds: &mut Vec<Command>) {
    let backup_new_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to create a new backup")
        .handler(backup_new);

    let backup_restore_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["restore"])
        .description("Command to apply the backup to the server")
        .handler(backup_restore);

    let backup_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["backup"])
        .description("Module to interact with the backup system")
        .group("Premium")
        .sub_command(backup_new_cmd)
        .sub_command(backup_restore_cmd)
        .handler(backup);
    cmds.push(backup_cmd);
}

#[derive(FromArgs)]
pub struct BackupArguments {
    pub name: String,
}

#[derive(FromArgs)]
pub struct BackupViewArguments {}

pub async fn backup(ctx: CommandContext, _args: BackupArguments) -> CommandResult {
    match ctx.bot.database.get_premium(ctx.author.id.0).await? {
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let backups = ctx.bot.database.get_backups(ctx.author.id.0).await?;
    let mut embed = EmbedBuilder::new().default_data().title("Backups").unwrap();

    for backup in backups {
        let val = format!("Prefix: {}\nVerification: {}\nVerified: {}\nRankbinds: {}\nGroupbinds: {}\nCustombinds: {}\nAssetbinds: {}",
            backup.command_prefix.unwrap_or_else(|| "!".into()), backup.verification_role.unwrap_or_default(), backup.verified_role.unwrap_or_default(),
            backup.rankbinds.len(), backup.groupbinds.len(), backup.custombinds.len(), backup.assetbinds.len()
        );
        embed = embed.field(EmbedFieldBuilder::new(backup.name, val).unwrap());
    }

    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed.build().unwrap())
        .unwrap()
        .await?;
    Ok(())
}
