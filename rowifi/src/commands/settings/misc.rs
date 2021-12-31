use rowifi_framework::prelude::*;
use rowifi_models::guild::BlacklistActionType;

use super::ToggleOption;

#[derive(FromArgs)]
pub struct BlacklistActionArguments {
    #[arg(
        help = "The action to be performed on detecting a blacklist. Must be one of `None` `Kick` `Ban`"
    )]
    pub option: BlacklistActionType,
}

pub async fn blacklist_action(
    ctx: CommandContext,
    args: BlacklistActionArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let bl_type = args.option;
    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET blacklist_action = $1 WHERE guild_id = $2",
            &[&bl_type, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "Blacklist action has successfully been set to {}",
            bl_type
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Blacklist Action - {} -> {}",
            guild.blacklist_action, bl_type
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct ToggleCommandsArguments {
    #[arg(
        help = "The toggle to enable/disable commands in the channel. Must be one of `enable` `disable` `on` `off`"
    )]
    pub option: ToggleOption,
}

pub async fn toggle_commands(ctx: CommandContext, args: ToggleCommandsArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let option = args.option;
    let (statement, desc, add) = match option {
        ToggleOption::Enable => (
            "UPDATE guilds SET disabled_channels = array_remove(disabled_channels, $1) WHERE guild_id = $2",
            "Commands have been successfully enabled in this channel",
            false,
        ),
        ToggleOption::Disable => (
            "UPDATE guilds SET disabled_channels = array_append(disabled_channels, $1) WHERE guild_id = $2",
            "Commands have been successfully disabled in this channel",
            true,
        ),
    };

    ctx.bot
        .database
        .execute(
            statement,
            &[&(ctx.channel_id.get() as i64), &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(desc)
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    if add {
        ctx.bot.disabled_channels.insert(ctx.channel_id);
    } else {
        ctx.bot.disabled_channels.remove(&ctx.channel_id);
    }
    Ok(())
}

#[derive(FromArgs)]
pub struct SettingsPrefixArguments {
    #[arg(help = "The string that is to be set as the bot's prefix in the server")]
    pub prefix: String,
}

pub async fn settings_prefix(ctx: CommandContext, args: SettingsPrefixArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let prefix = args.prefix;
    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET command_prefix = $1 WHERE guild_id = $2",
            &[&prefix, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "The bot prefix has been successfully changed to {}",
            prefix
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    ctx.bot.prefixes.insert(guild_id, prefix);
    Ok(())
}
