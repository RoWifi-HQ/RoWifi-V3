mod new;

use crate::framework::prelude::*;
use twilight_embed_builder::EmbedFieldBuilder;

pub use new::*;

pub static BACKUP_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["backup"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&BACKUP_NEW_COMMAND]
};

pub static BACKUP_COMMAND: Command = Command {
    fun: backup,
    options: &BACKUP_OPTIONS
};

#[command]
pub async fn backup(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let backups = ctx.database.get_backups(msg.author.id.0).await?;
    let mut embed = EmbedBuilder::new().default_data()
        .title("Backups").unwrap();

    for backup in backups {
        let val = format!("Prefix: {}\nVerification: {}\nVerified: {}\nRankbinds: {}\nGroupbinds: {}\nCustombinds: {}\nAssetbinds: {}",
            backup.command_prefix.unwrap_or("!".into()), backup.verification_role.unwrap_or_default(), backup.verified_role.unwrap_or_default(),
            backup.rankbinds.len(), backup.groupbinds.len(), backup.custombinds.len(), backup.assetbinds.len()
        );
        embed = embed.field(EmbedFieldBuilder::new(backup.name, val).unwrap());
    }

    let _ = ctx.http.create_message(msg.channel_id).embed(embed.build().unwrap()).unwrap().await?;
    Ok(())
}