use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::GuildType,
    user::{RoUser, UserFlags},
};

pub async fn premium_patreon(ctx: CommandContext) -> CommandResult {
    let author = ctx.author.id.0;
    let user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(author.get() as i64)],
        )
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Patreon Linking Failed")
                .description("You need to be verified with RoWifi to redeem your premium. Please do so using `/verify`")
                .build()?;
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let (premium_id, tier) = ctx.bot.patreon.get_patron(author.get()).await?;
    let premium_id = match premium_id.map(|p| p.parse::<i64>()) {
        Some(Ok(p)) => p,
        _ => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Patreon Linking Failed")
                .description("Patreon Account was not found for this Discord Account. Please make sure your Discord Account is linked to your patreon account")
                .build()?;
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let tier = match tier.map(|t| t.parse::<i64>()) {
        Some(Ok(t)) => t,
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Patreon Linking Failed")
                .description("You were not found to be a member of any tier")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let transferred_user = match user.transferred_to {
        Some(t) => {
            ctx.bot
                .database
                .query_opt::<RoUser>("SELECT * FROM users WHERE discord_id = $1", &[&t])
                .await?
        }
        None => None,
    };

    // At this point, there's only two things that have happened, premium_id changed or tier changed
    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    // If the premium id has changed
    if let Some(patreon_id) = user.patreon_id {
        if patreon_id != premium_id {
            let premium_changed = transaction
                .prepare_cached("UPDATE users SET patreon_id = $1 WHERE discord_id = $2")
                .await?;
            transaction
                .execute(&premium_changed, &[&premium_id, &(author.get() as i64)])
                .await?;
        }
    }

    let tier_changed = transaction
        .prepare_cached("UPDATE users SET flags = $1 WHERE discord_id = $2")
        .await?;

    if tier == 4_014_582 {
        if !user.flags.contains(UserFlags::ALPHA) {
            let mut new_flags = user.flags;
            new_flags.remove(UserFlags::BETA);
            new_flags.insert(UserFlags::ALPHA);
            transaction
                .execute(&tier_changed, &[&new_flags, &(author.get() as i64)])
                .await?;

            if let Some(transferred_to) = &transferred_user {
                let mut new_flags = transferred_to.flags;
                new_flags.remove(UserFlags::BETA);
                new_flags.insert(UserFlags::ALPHA);
                transaction
                    .execute(&tier_changed, &[&new_flags, &transferred_to.discord_id])
                    .await?;
            }
        }
    } else if tier == 4_656_839 {
        if !user.flags.contains(UserFlags::BETA) {
            let mut new_flags = user.flags;
            new_flags.remove(UserFlags::ALPHA);
            new_flags.insert(UserFlags::BETA);
            transaction
                .execute(&tier_changed, &[&new_flags, &(author.get() as i64)])
                .await?;

            if let Some(transferred_to) = &transferred_user {
                let mut new_flags = transferred_to.flags;
                new_flags.remove(UserFlags::ALPHA);
                new_flags.insert(UserFlags::BETA);
                transaction
                    .execute(&tier_changed, &[&new_flags, &transferred_to.discord_id])
                    .await?;
            }

            let servers = transferred_user
                .map_or_else(|| user.premium_servers.clone(), |t| t.premium_servers);
            let guild_change = transaction
                .prepare_cached("UPDATE guilds SET kind = $1 WHERE guild_id = $2")
                .await?;
            for server in servers {
                transaction
                    .execute(&guild_change, &[&GuildType::Beta, &server])
                    .await?;
            }
        }
    } else {
        return Ok(());
    }
    transaction.commit().await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Patreon Linking Successful")
        .description("Your patreon account has successfully been registered with our database")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}
