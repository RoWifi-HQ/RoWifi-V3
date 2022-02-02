use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::gateway::payload::outgoing::RequestGuildMembers,
    guild::GuildType,
    id::UserId,
    user::{RoUser, UserFlags},
};
use std::env;

pub async fn premium_redeem(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(ctx.author.id.get() as i64)],
        )
        .await?
    {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Premium Redeem Failed")
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details")
                .build()?;
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if premium_user.transferred_to.is_some() {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
            .title("Premium Redeem Failed")
            .description("You seem to have transferred your premium to someone else. Thus, you cannot use it.")
            .build()?;
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let author_id = UserId(ctx.author.id);
    if !(server.owner_id == author_id || ctx.bot.owners.contains(&author_id)) {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Redeem Failed")
            .description("You must be the server owner to redeem premium in a server")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    if premium_user.flags.contains(UserFlags::ALPHA) {
        if !premium_user.premium_servers.is_empty() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Redeem Failed")
                .description("You may only use premium in one of your servers")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    }

    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let guild_change = transaction
        .prepare_cached("UPDATE guilds SET kind = $1, premium_owner = $2 WHERE guild_id = $3")
        .await?;
    let guild_type = if premium_user.flags.contains(UserFlags::ALPHA) {
        GuildType::Alpha
    } else if premium_user.flags.contains(UserFlags::BETA) {
        GuildType::Beta
    } else {
        return Ok(());
    };
    transaction
        .execute(
            &guild_change,
            &[&guild_type, &(ctx.author.id.get() as i64), &guild.guild_id],
        )
        .await?;

    if !premium_user.premium_servers.contains(&(guild_id)) {
        let user_change = transaction.prepare_cached("UPDATE users SET premium_servers = array_append(premium_servers, $1) WHERE discord_id = $2").await?;
        transaction
            .execute(&user_change, &[&(guild_id), &(ctx.author.id.get() as i64)])
            .await?;
    }
    transaction.commit().await?;

    ctx.bot
        .admin_roles
        .insert(guild_id, guild.admin_roles.clone());
    ctx.bot
        .trainer_roles
        .insert(guild_id, guild.trainer_roles.clone());
    ctx.bot
        .bypass_roles
        .insert(guild_id, guild.bypass_roles.clone());
    ctx.bot
        .nickname_bypass_roles
        .insert(guild_id, guild.nickname_bypass_roles.clone());

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Premium Redeem Successful")
        .description(format!("Added Premium Features to {}", server.name))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let req = RequestGuildMembers::builder(server.id.0).query("", None);
    let total_shards = env::var("TOTAL_SHARDS").unwrap().parse::<u64>().unwrap();
    let shard_id = (guild_id.0.get() >> 22) % total_shards;
    let _res = ctx.bot.cluster.command(shard_id, &req).await;
    Ok(())
}

pub async fn premium_remove(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx
        .bot
        .database
        .query_opt::<RoUser>(
            "SELECT * FROM users WHERE discord_id = $1",
            &[&(ctx.author.id.get() as i64)],
        )
        .await?
    {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Premium Disable Failed")
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details")
                .build()?;
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    if !premium_user.premium_servers.contains(&(guild_id)) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
            .title("Premium Disable Failed")
            .description("This server either does not have premium enabled or the premium is owned by an another member")
            .build()?;
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let guild_change = transaction
        .prepare_cached("UPDATE guilds SET kind = $1, premium_owner = NULL, auto_detection = false WHERE guild_id = $2")
        .await?;
    transaction
        .execute(&guild_change, &[&GuildType::Free, &guild.guild_id])
        .await?;

    let user_change = transaction.prepare_cached("UPDATE users SET premium_servers = array_remove(premium_servers, $1) WHERE discord_id = $2").await?;
    transaction
        .execute(
            &user_change,
            &[&guild.guild_id, &(ctx.author.id.get() as i64)],
        )
        .await?;

    transaction.commit().await?;

    ctx.bot.admin_roles.remove(&guild_id);
    ctx.bot.trainer_roles.remove(&guild_id);
    ctx.bot.bypass_roles.remove(&guild_id);
    ctx.bot.nickname_bypass_roles.remove(&guild_id);

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Premium Disable Successful")
        .description(format!("Removed Premium Features from {}", server.name))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
