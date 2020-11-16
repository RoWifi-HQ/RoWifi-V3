mod admin;
mod patreon;
mod redeem;
mod transfer;

use crate::framework::prelude::*;
use crate::models::user::PremiumType;
use twilight_model::id::UserId;

use admin::{PREMIUM_ADD_COMMAND, PREMIUM_CHECK_COMMAND, PREMIUM_DELETE_COMMAND};
use patreon::PREMIUM_PATREON_COMMAND;
use redeem::{PREMIUM_REDEEM_COMMAND, PREMIUM_REMOVE_COMMAND};
use transfer::PREMIUM_TRANSFER_COMMAND;

pub static PREMIUM_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["premium"],
    desc: Some("Command to view the premium status about an user"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &PREMIUM_PATREON_COMMAND,
        &PREMIUM_REDEEM_COMMAND,
        &PREMIUM_ADD_COMMAND,
        &PREMIUM_DELETE_COMMAND,
        &PREMIUM_REMOVE_COMMAND,
        &PREMIUM_TRANSFER_COMMAND,
        &PREMIUM_CHECK_COMMAND,
    ],
    group: Some("Premium"),
};

pub static PREMIUM_COMMAND: Command = Command {
    fun: premium,
    options: &PREMIUM_OPTIONS,
};

#[command]
pub async fn premium(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let author = match args
        .next()
        .and_then(parse_username)
        .and_then(|u| ctx.cache.user(UserId(u)))
    {
        Some(a) => (a.id, a.name.clone(), a.discriminator.clone()),
        None => (
            msg.author.id,
            msg.author.name.clone(),
            msg.author.discriminator.clone(),
        ),
    };
    let mut embed = EmbedBuilder::new()
        .default_data()
        .title(format!("{}#{}", author.1, author.2))
        .unwrap();
    if let Some(premium_user) = ctx.database.get_premium((author.0).0).await? {
        embed = match premium_user.premium_type {
            PremiumType::Beta => embed.field(EmbedFieldBuilder::new("Tier", "Beta").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System (Upcoming)").unwrap()),
            PremiumType::Alpha => embed.field(EmbedFieldBuilder::new("Tier", "Alpha").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for one owned server\nUpdate All/Update Role (3 times per 12 hours)").unwrap()),
            PremiumType::Partner => embed.field(EmbedFieldBuilder::new("Tier", "Partner").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System (Upcoming)").unwrap()),
            PremiumType::Council => embed.field(EmbedFieldBuilder::new("Tier", "Council").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System (Upcoming)").unwrap()),
            PremiumType::Staff => embed.field(EmbedFieldBuilder::new("Tier", "Staff").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for one owned server\nUpdate All/Update Role (3 times per 12 hours)").unwrap()),
        };
    } else {
        embed = embed
            .field(EmbedFieldBuilder::new("Tier", "Normal").unwrap())
            .field(EmbedFieldBuilder::new("Perks", "None").unwrap());
    }
    let embed = embed.build().unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    Ok(())
}
