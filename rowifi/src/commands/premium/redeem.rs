use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::{gateway::payload::outgoing::RequestGuildMembers, id::RoleId},
    guild::GuildType,
};
use std::env;

pub async fn premium_redeem(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx.bot.database.get_premium(ctx.author.id.0.get()).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Premium Redeem Failed")
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details")
                .build().unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    if !(server.owner_id == ctx.author.id || ctx.bot.owners.contains(&ctx.author.id)) {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Premium Redeem Failed")
            .description("You must be the server owner to redeem premium in a server")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let guild_type: GuildType = premium_user.premium_type.into();
    if let GuildType::Alpha = guild_type {
        if !premium_user.discord_servers.is_empty() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Premium Redeem Failed")
                .description("You may only use premium in one of your servers")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    }

    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let filter = doc! {"_id": guild_id.0.get() as i64};
    let update =
        doc! {"$set": {"Settings.Type": guild_type as i32, "Settings.AutoDetection": true}};
    ctx.bot.database.modify_guild(filter, update).await?;

    if !premium_user.discord_servers.contains(&(guild_id.0.get() as i64)) {
        let filter2 = doc! {"_id": ctx.author.id.0.get() as i64};
        let update2 = doc! {"$push": { "Servers": guild_id.0.get() as i64 }};
        ctx.bot.database.modify_premium(filter2, update2).await?;
    }

    ctx.bot.admin_roles.insert(
        guild_id,
        guild
            .settings
            .admin_roles
            .iter()
            .map(|r| RoleId::new(*r as u64).unwrap())
            .collect(),
    );
    ctx.bot.trainer_roles.insert(
        guild_id,
        guild
            .settings
            .trainer_roles
            .iter()
            .map(|r| RoleId::new(*r as u64).unwrap())
            .collect(),
    );
    ctx.bot.bypass_roles.insert(
        guild_id,
        guild
            .settings
            .bypass_roles
            .iter()
            .map(|r| RoleId::new(*r as u64).unwrap())
            .collect(),
    );
    ctx.bot.nickname_bypass_roles.insert(
        guild_id,
        guild
            .settings
            .nickname_bypass_roles
            .iter()
            .map(|r| RoleId::new(*r as u64).unwrap())
            .collect(),
    );

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Premium Redeem Successful")
        .description(format!("Added Premium Features to {}", server.name))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed]).exec().await?;

    let req = RequestGuildMembers::builder(server.id).query("", None);
    let total_shards = env::var("TOTAL_SHARDS").unwrap().parse::<u64>().unwrap();
    let shard_id = (guild_id.0.get() >> 22) % total_shards;
    let _res = ctx.bot.cluster.command(shard_id, &req).await;
    Ok(())
}

pub async fn premium_remove(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx.bot.database.get_premium(ctx.author.id.0.get()).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
                .title("Premium Disable Failed")
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details")
                .build().unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };

    if !premium_user.discord_servers.contains(&(guild_id.0.get() as i64)) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
            .title("Premium Disable Failed")
            .description("This server either does not have premium enabled or the premium is owned by an another member")
            .build().unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let filter = doc! {"_id": guild_id.0.get() as i64};
    let update =
        doc! {"$set": {"Settings.Type": GuildType::Normal as i32, "Settings.AutoDetection": false}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let filter2 = doc! {"_id": ctx.author.id.0.get() as i64};
    let update2 = doc! {"$pull": { "Servers": guild_id.0.get() as i64 }};
    ctx.bot.database.modify_premium(filter2, update2).await?;

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
    ctx.respond().embeds(&[embed]).exec().await?;

    Ok(())
}
