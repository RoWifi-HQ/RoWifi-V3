use rowifi_framework::prelude::*;
use rowifi_models::user::PremiumUser;
use twilight_model::id::UserId;

#[derive(FromArgs)]
pub struct PremiumTransferArguments {
    #[arg(help = "The Discord User to who you want to transfer your premium to")]
    pub user_id: Option<UserId>,
}

pub async fn premium_transfer(
    ctx: CommandContext,
    args: PremiumTransferArguments,
) -> CommandResult {
    let premium_user = ctx.bot.database.get_premium(ctx.author.id.0).await?;
    if let Some(premium_user) = premium_user {
        if premium_user.premium_owner.is_some() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Premium Transfer Failed")
                .unwrap()
                .description("You may not transfer a premium that you do not own")
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
        let to_transfer_id = match args.user_id {
            Some(s) => s,
            None => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .unwrap()
                    .title("Premium Transfer Failed")
                    .unwrap()
                    .description("You must specify a user id to transfer to")
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
        if ctx
            .bot
            .database
            .get_premium(to_transfer_id.0 as u64)
            .await?
            .is_some()
        {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Premium Transfer Failed")
                .unwrap()
                .description("You cannot transfer premium to a user who already has premium")
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

        let new_premium_user = PremiumUser {
            discord_id: to_transfer_id.0 as i64,
            patreon_id: None,
            discord_servers: Vec::new(),
            premium_type: premium_user.premium_type,
            premium_owner: Some(premium_user.discord_id),
            premium_patreon_owner: premium_user.patreon_id,
        };
        ctx.bot.database.delete_premium(ctx.author.id.0).await?;
        ctx.bot
            .database
            .add_premium(new_premium_user, false)
            .await?;

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .unwrap()
            .title("Premium Transfer Successful")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    } else if let Some(transferred_premium_user) = ctx
        .bot
        .database
        .get_transferred_premium(ctx.author.id.0)
        .await?
    {
        ctx.bot
            .database
            .delete_premium(transferred_premium_user.discord_id as u64)
            .await?;

        let premium_user = PremiumUser {
            discord_id: ctx.author.id.0 as i64,
            patreon_id: transferred_premium_user.premium_patreon_owner,
            discord_servers: Vec::new(),
            premium_type: transferred_premium_user.premium_type,
            premium_owner: None,
            premium_patreon_owner: None,
        };
        ctx.bot.database.add_premium(premium_user, false).await?;

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .unwrap()
            .title("Premium Transfer Successful")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    } else {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Premium Transfer Failed")
            .unwrap()
            .description("You do not have a premium subscription")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
    }
    Ok(())
}
