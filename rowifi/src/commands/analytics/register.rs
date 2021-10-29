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
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Group Registration Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let group_id = args.group_id;
    if guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration Already Exists")
            .color(Color::Red as u32)
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"RegisteredGroups": group_id}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Group Registration Successful")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed]).exec().await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct UnregisterArguments {
    #[arg(help = "Group Id that is to be unregistered")]
    pub group_id: i64,
}

pub async fn analytics_unregister(ctx: CommandContext, args: UnregisterArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Group Registration Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let group_id = args.group_id;
    if !guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration doesn't exist")
            .color(Color::Red as u32)
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"RegisteredGroups": group_id}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Group Unregistration Successful")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed]).exec().await?;
    Ok(())
}
