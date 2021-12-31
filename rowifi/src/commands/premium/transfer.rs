use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::GuildType,
    id::UserId,
    user::{RoUser, UserFlags},
};

#[derive(FromArgs)]
pub struct PremiumTransferArguments {
    #[arg(help = "The Discord User to who you want to transfer your premium to")]
    pub user_id: Option<UserId>,
}

pub async fn premium_transfer(
    ctx: CommandContext,
    args: PremiumTransferArguments,
) -> CommandResult {
    let user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(ctx.author.id.get() as i64)],
        )
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Transfer Failed")
                .description("You must be verified to use this command")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if user.transferred_to.is_some() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Transfer Failed")
            .description("You have already transferred a premium to someone else. You may not transfer it again.")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    if user.transferred_from.is_some() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Transfer Failed")
            .description("You may not transfer a premium that you do not own")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let to_transfer_id = match args.user_id {
        Some(s) => s,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Transfer Failed")
                .description("You must specify a user id to transfer to.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if to_transfer_id.0 == ctx.author.id {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Transfer Failed")
            .description("You cannot transfer your premium to yourself.")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let transfer_to_user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(to_transfer_id.get() as i64)],
        )
        .await?
    {
        Some(t) => t,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Transfer Failed")
                .description("The user you are transferring to must also be verified")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if transfer_to_user
        .flags
        .contains(UserFlags::ALPHA | UserFlags::BETA)
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Transfer Failed")
            .description("You cannot transfer to a user who already has premium")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let guild_change = transaction
        .prepare_cached("UPDATE guilds SET kind = $1 WHERE guild_id = $2")
        .await?;
    for server in user.premium_servers {
        transaction
            .execute(&guild_change, &[&GuildType::Free, &server])
            .await?;
    }

    let transferrer_change = transaction
        .prepare_cached(
            "UPDATE users SET premium_servers = $1, transferred_to = $2 WHERE discord_id = $3",
        )
        .await?;
    transaction
        .execute(
            &transferrer_change,
            &[
                &Vec::<i64>::new(),
                &transfer_to_user.discord_id,
                &user.discord_id,
            ],
        )
        .await?;

    let mut transferee_flags = transfer_to_user.flags;
    if user.flags.contains(UserFlags::ALPHA) {
        transferee_flags.insert(UserFlags::ALPHA);
    } else if user.flags.contains(UserFlags::BETA) {
        transferee_flags.insert(UserFlags::BETA);
    }
    let transferee_change = transaction
        .prepare_cached("UPDATE users SET flags = $1, transferred_from = $2 WHERE discord_id = $3")
        .await?;
    transaction
        .execute(
            &transferee_change,
            &[
                &transferee_flags,
                &user.discord_id,
                &transfer_to_user.discord_id,
            ],
        )
        .await?;

    transaction.commit().await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Premium Transfer Successful")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

pub async fn premium_untransfer(ctx: CommandContext) -> CommandResult {
    let user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(ctx.author.id.get() as i64)],
        )
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Transfer Failed")
                .description("You must be verified to use this command")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if user.transferred_to.is_none() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Transfer Failed")
            .description("You have not transferred your premium to anyone.")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let transfer_to_user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&user.transferred_to.unwrap()],
        )
        .await?
    {
        Some(t) => t,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Transfer Failed")
                .description("The user you have to transferred to doesn't exist. This shouldn't happen. Please contact the RoWifi support server.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let guild_change = transaction
        .prepare_cached("UPDATE guilds SET kind = $1 WHERE guild_id = $2")
        .await?;
    for server in transfer_to_user.premium_servers {
        transaction
            .execute(&guild_change, &[&GuildType::Free, &server])
            .await?;
    }

    let mut transferee_flags = transfer_to_user.flags;
    transferee_flags.remove(UserFlags::ALPHA);
    transferee_flags.remove(UserFlags::BETA);
    let transferee_change = transaction.prepare_cached("UPDATE users SET flags = $1, transferred_from = NULL, premium_servers = $2 WHERE discord_id = $3").await?;
    transaction
        .execute(
            &transferee_change,
            &[
                &transferee_flags,
                &Vec::<i64>::new(),
                &transfer_to_user.discord_id,
            ],
        )
        .await?;

    let transferrer_change = transaction
        .prepare_cached("UPDATE users SET transferred_to = NULL WHERE discord_id = $1")
        .await?;
    transaction
        .execute(&transferrer_change, &[&user.discord_id])
        .await?;

    transaction.commit().await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Premium Untransfer Successful")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
