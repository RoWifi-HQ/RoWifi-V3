use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

#[derive(FromArgs)]
pub struct RegisterArguments {
    #[arg(help = "Group Id that is to be registered")]
    pub group_id: i64,
}

pub async fn analytics_register(ctx: CommandContext, args: RegisterArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Group Registration Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
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

    let group_id = args.group_id;
    if guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration Already Exists")
            .unwrap()
            .color(Color::Red as u32)
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

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"RegisteredGroups": group_id}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Group Registration Successful")
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

#[derive(FromArgs)]
pub struct UnregisterArguments {
    #[arg(help = "Group Id that is to be unregistered")]
    pub group_id: i64,
}

pub async fn analytics_unregister(ctx: CommandContext, args: UnregisterArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Group Registration Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let group_id = args.group_id;
    if !guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration doesn't exist")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"RegisteredGroups": group_id}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Group Unregistration Successful")
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    Ok(())
}
