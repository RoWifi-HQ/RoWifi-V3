mod new;
mod restore;

use rowifi_framework::prelude::*;
use rowifi_models::{user::{RoUser, UserFlags}, guild::backup::GuildBackup, bind::BindType};

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

pub async fn backup(ctx: CommandContext) -> CommandResult {
    let user = match ctx.bot.database.query_opt::<RoUser>("SELECT * FROM users WHERE discord_id = $1", &[&(ctx.author.id.get() as i64)]).await? {
        Some(u) if u.flags.contains(UserFlags::BETA) => u,
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Backup Failed")
                .description("This module may only be used by a Beta Tier user")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let backups = ctx.bot.database.query::<GuildBackup>("SELECT * FROM backups WHERE user_id = $1", &[&user.discord_id]).await?;
    let mut embed = EmbedBuilder::new().default_data().title("Backups");

    for backup in backups {
        let data = backup.data.0;
        let r = data.binds.iter().map(|b| b.kind() == BindType::Rank).count();
        let g = data.binds.iter().map(|b| b.kind() == BindType::Group).count();
        let c = data.binds.iter().map(|b| b.kind() == BindType::Custom).count();
        let a = data.binds.iter().map(|b| b.kind() == BindType::Asset).count();
        let val = format!("Prefix: {}\nVerification: {:?}\nVerified: {:?}\nRankbinds: {}\nGroupbinds: {}\nCustombinds: {}\nAssetbinds: {}",
            data.command_prefix, data.verification_roles.get(0), data.verified_roles.get(0),
            r, g, c, a
        );
        embed = embed.field(EmbedFieldBuilder::new(backup.name, val));
    }

    ctx.respond().embeds(&[embed.build()?])?.exec().await?;
    Ok(())
}
