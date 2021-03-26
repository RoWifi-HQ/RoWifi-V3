use rowifi_framework::prelude::*;
use rowifi_models::user::{PremiumType, PremiumUser};

#[derive(FromArgs)]
pub struct PremiumPatreonArguments {}

pub async fn premium_patreon(ctx: CommandContext, _args: PremiumPatreonArguments) -> CommandResult {
    let author = ctx.author.id.0;
    let premium_already = ctx.bot.database.get_premium(author).await?.is_some();
    let premium_user: PremiumUser;
    let (patreon_id, tier) = ctx.bot.patreon.get_patron(author).await?;
    if patreon_id.is_none() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Patreon Linking Failed").unwrap()
            .description("Patreon Account was not found for this Discord Account. Please make sure your Discord Account is linked to your patreon account").unwrap()
            .build().unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    if tier.is_none() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Patreon Linking Failed")
            .unwrap()
            .description("You were not found to be a member of any tier")
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

    let patreon_id = patreon_id.unwrap().parse::<i64>().unwrap();
    let tier = tier.unwrap().parse::<i64>().unwrap();
    if tier == 4_014_582 {
        premium_user = PremiumUser {
            discord_id: author as i64,
            patreon_id: Some(patreon_id),
            premium_type: PremiumType::Alpha,
            discord_servers: Vec::new(),
            premium_owner: None,
            premium_patreon_owner: None,
        };
    } else if tier == 4_656_839 {
        premium_user = PremiumUser {
            discord_id: author as i64,
            patreon_id: Some(patreon_id),
            premium_type: PremiumType::Beta,
            discord_servers: Vec::new(),
            premium_owner: None,
            premium_patreon_owner: None,
        };
    } else {
        return Ok(());
    }

    let transferred_premium = ctx.bot.database.get_transferred_premium(author).await?;
    if let Some(transferred_premium) = transferred_premium {
        ctx.bot.database.delete_premium(transferred_premium.discord_id as u64).await?;
    }
    
    ctx.bot
        .database
        .add_premium(premium_user, premium_already)
        .await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Patreon Linking Successful")
        .unwrap()
        .description("Your patreon account has successfully been registered with our database")
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
