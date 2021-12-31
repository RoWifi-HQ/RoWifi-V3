use rowifi_framework::prelude::*;
use rowifi_models::{guild::GuildType, id::ChannelId};

#[derive(FromArgs)]
pub struct LogChannelArguments {
    #[arg(help = "The channel to set for logs generated by RoWifi")]
    pub channel: Option<ChannelId>,
}

pub async fn log_channel(ctx: CommandContext, args: LogChannelArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    if guild.kind == GuildType::Free {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    if let Some(channel_id) = args.channel {
        if ctx.bot.cache.channel(channel_id).is_none() {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Command Failed")
                .description("This channel cannot be set as a log channel or does not exist")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }

        ctx.bot
            .database
            .execute(
                "UPDATE guilds SET log_channel = $1 WHERE guild_id = $2",
                &[&(channel_id.get() as i64), &guild.guild_id],
            )
            .await?;
        ctx.bot.log_channels.insert(guild_id, channel_id);

        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::DarkGreen as u32)
            .title("Settings Modification Successful")
            .description(format!("Logs channel has been set to <#{}>", channel_id))
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
    } else if let Some(channel_id) = guild.log_channel {
        ctx.respond()
            .content(&format!("Current log channel is <#{}>", channel_id))?
            .exec()
            .await?;
    } else {
        ctx.respond()
            .content("This server does not have a log channel set up")?
            .exec()
            .await?;
    }

    Ok(())
}
