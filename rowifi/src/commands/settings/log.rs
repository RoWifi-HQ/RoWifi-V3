use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

#[derive(FromArgs)]
pub struct LogChannelArguments {
    pub channel: Option<i64>,
}

pub async fn log_channel(ctx: CommandContext, args: LogChannelArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    if let Some(channel_id) = args.channel {
        let filter = doc! {"_id": guild.id};
        let update = doc! {"$set": {"LogChannel": channel_id}};
        ctx.bot.database.modify_guild(filter, update).await?;

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .title("Settings Modification Successful")
            .description(format!("Logs channel has been set to <#{}>", channel_id))
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
    } else if let Some(channel_id) = guild.settings.log_channel {
        ctx.respond()
            .content(format!("Current log channel is <#{}>", channel_id))
            .await?;
    } else {
        ctx.respond()
            .content("This server does not have a log channel set up")
            .await?;
    }

    Ok(())
}
