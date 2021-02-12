use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;
use std::env;
use twilight_model::gateway::payload::RequestGuildMembers;

#[derive(FromArgs)]
pub struct PremiumArguments {}

pub async fn premium_redeem(ctx: CommandContext, _args: PremiumArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx.bot.database.get_premium(ctx.author.id.0).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                .title("Premium Redeem Failed").unwrap()
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details").unwrap()
                .build().unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    if !(server.owner_id == ctx.author.id || ctx.bot.owners.contains(&ctx.author.id)) {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Premium Redeem Failed")
            .unwrap()
            .description("You must be the server owner to redeem premium in a server")
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

    let guild_type: GuildType = premium_user.premium_type.into();
    if let GuildType::Alpha = guild_type {
        if !premium_user.discord_servers.is_empty() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Premium Redeem Failed")
                .unwrap()
                .description("You may only use premium in one of your servers")
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
    }

    ctx.bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let filter = doc! {"_id": guild_id.0};
    let update =
        doc! {"$set": {"Settings.Type": guild_type as i32, "Settings.AutoDetection": true}};
    ctx.bot.database.modify_guild(filter, update).await?;

    if !premium_user.discord_servers.contains(&(guild_id.0 as i64)) {
        let filter2 = doc! {"_id": ctx.author.id.0};
        let update2 = doc! {"$push": { "Servers": guild_id.0 }};
        ctx.bot.database.modify_premium(filter2, update2).await?;
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Premium Redeem Successful")
        .unwrap()
        .description(format!("Added Premium Features to {}", server.name))
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let req = RequestGuildMembers::builder(server.id).query("", None);
    let total_shards = env::var("TOTAL_SHARDS").unwrap().parse::<u64>().unwrap();
    let shard_id = (guild_id.0 >> 22) % total_shards;
    let _res = ctx.bot.cluster.command(shard_id, &req).await;
    Ok(())
}

pub async fn premium_remove(ctx: CommandContext, _args: PremiumArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let premium_user = match ctx.bot.database.get_premium(ctx.author.id.0).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                .title("Premium Disable Failed").unwrap()
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details").unwrap()
                .build().unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    if !premium_user.discord_servers.contains(&(guild_id.0 as i64)) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Premium Disable Failed").unwrap()
            .description("This server either does not have premium enabled or the premium is owned by an another member").unwrap()
            .build().unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    ctx.bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let filter = doc! {"_id": guild_id.0};
    let update =
        doc! {"$set": {"Settings.Type": GuildType::Normal as i32, "Settings.AutoDetection": false}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let filter2 = doc! {"_id": ctx.author.id.0};
    let update2 = doc! {"$pull": { "Servers": guild_id.0 }};
    ctx.bot.database.modify_premium(filter2, update2).await?;

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Premium Disable Successful")
        .unwrap()
        .description(format!("Removed Premium Features from {}", server.name))
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
