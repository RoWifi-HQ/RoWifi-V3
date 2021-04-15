mod patreon;
mod redeem;
mod transfer;

use rowifi_framework::prelude::*;
use rowifi_models::user::PremiumType;
use twilight_model::id::UserId;

use self::patreon::premium_patreon;
use redeem::{premium_redeem, premium_remove};
use transfer::premium_transfer;

pub fn premium_config(cmds: &mut Vec<Command>) {
    let premium_redeem_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["redeem"])
        .description("Command to redeem premium in a server")
        .handler(premium_redeem);

    let premium_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to disable premium from the server")
        .handler(premium_remove);

    let premium_patreon_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["patreon"])
        .description("Command to link your patreon account to your discord account")
        .handler(premium_patreon);

    let premium_transfer_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["transfer"])
        .description("Command to transfer your premium to another account")
        .handler(premium_transfer);

    let premium_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["premium"])
        .description("Module to interact with the premium subsystem")
        .group("Premium")
        .sub_command(premium_patreon_cmd)
        .sub_command(premium_redeem_cmd)
        .sub_command(premium_remove_cmd)
        .sub_command(premium_transfer_cmd)
        .handler(premium);

    cmds.push(premium_cmd);
}

#[derive(FromArgs)]
pub struct PremiumViewArguments {
    pub user_id: Option<UserId>,
}

pub async fn premium(ctx: CommandContext, args: PremiumViewArguments) -> CommandResult {
    let author = match args.user_id.and_then(|u| ctx.bot.cache.user(u)) {
        Some(a) => (a.id, a.name.clone(), a.discriminator.clone()),
        None => (
            ctx.author.id,
            ctx.author.name.clone(),
            ctx.author.discriminator.clone(),
        ),
    };
    let mut embed = EmbedBuilder::new()
        .default_data()
        .title(format!("{}#{}", author.1, author.2))
        .unwrap();
    if let Some(premium_user) = ctx.bot.database.get_premium((author.0).0).await? {
        embed = match premium_user.premium_type {
            PremiumType::Beta => embed.field(EmbedFieldBuilder::new("Tier", "Beta").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System").unwrap()),
            PremiumType::Alpha => embed.field(EmbedFieldBuilder::new("Tier", "Alpha").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for one owned server\nUpdate All/Update Role (3 times per 12 hours)").unwrap()),
            PremiumType::Partner => embed.field(EmbedFieldBuilder::new("Tier", "Partner").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System").unwrap()),
            PremiumType::Council => embed.field(EmbedFieldBuilder::new("Tier", "Council").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for all owned servers\nUpdate All/Update Role (3 times per 12 hours)\nBackups\nAnalytics\nEvent Logging System").unwrap()),
            PremiumType::Staff => embed.field(EmbedFieldBuilder::new("Tier", "Staff").unwrap())
                                    .field(EmbedFieldBuilder::new("Perks", "Auto Detection for one owned server\nUpdate All/Update Role (3 times per 12 hours)").unwrap()),
        };
    } else {
        embed = embed
            .field(EmbedFieldBuilder::new("Tier", "Normal").unwrap())
            .field(EmbedFieldBuilder::new("Perks", "None").unwrap());
    }
    ctx.respond().embed(embed.build().unwrap()).await?;

    Ok(())
}
