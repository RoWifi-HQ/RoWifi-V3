use crate::framework::prelude::*;
use crate::models::guild::BlacklistActionType;

pub static BLACKLIST_ACTION_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["blacklist-action", "bl-action"],
    desc: Some("Command to change the blacklist action"),
    usage: Some("settings blacklist-action <None/Kick/Ban>"),
    examples: &[],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static TOGGLE_COMMANDS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["commands", "command", "command-channel"],
    desc: Some("Command to disable/enable commands in a channel"),
    usage: Some("settings commands <enable/disable/on/off>"),
    examples: &[],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static SETTINGS_PREFIX_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["prefix"],
    desc: Some("Command to change the bot prefix"),
    usage: Some("settings prefix <NewPrefix>"),
    examples: &[],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static BLACKLIST_ACTION_COMMAND: Command = Command {
    fun: blacklist_action,
    options: &BLACKLIST_ACTION_OPTIONS,
};

pub static TOGGLE_COMMANDS_COMMAND: Command = Command {
    fun: toggle_commands,
    options: &TOGGLE_COMMANDS_OPTIONS,
};

pub static SETTINGS_PREFIX_COMMAND: Command = Command {
    fun: settings_prefix,
    options: &SETTINGS_PREFIX_OPTIONS,
};

#[command]
pub async fn blacklist_action(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(()),
    };
    let bl_type = match option.to_lowercase().as_str() {
        "none" => BlacklistActionType::None,
        "kick" => BlacklistActionType::Kick,
        "ban" => BlacklistActionType::Ban,
        _ => {
            return Err(CommandError::ParseArgument(
                option,
                "Blacklist Action".into(),
                "None/Ban/Kick".into(),
            )
            .into())
        }
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.BlacklistAction": bl_type as u32}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(format!(
            "Blacklist action has successfully been set to {}",
            bl_type
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description(format!(
            "Settings Modification: Blacklist Action - {} -> {}",
            guild.settings.blacklist_action, bl_type
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

#[command]
pub async fn toggle_commands(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(()),
    };

    let (update, desc, add) = match option.to_lowercase().as_str() {
        "on" | "enable" => (
            bson::doc! {"$pull": {"DisabledChannels": msg.channel_id.0}},
            "Commands have been successfully enabled in this channel",
            false,
        ),
        "off" | "disable" => (
            bson::doc! {"$push": {"DisabledChannels": msg.channel_id.0}},
            "Commands have been successfully disabled in this channel",
            true,
        ),
        _ => {
            return Err(CommandError::ParseArgument(
                option,
                "toggle".into(),
                "`enable`, `disable`, `on`, `off`".into(),
            )
            .into())
        }
    };
    let filter = bson::doc! {"_id": guild.id};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(desc)
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    if add {
        ctx.config.disabled_channels.insert(msg.channel_id);
    } else {
        ctx.config.disabled_channels.remove(&msg.channel_id);
    }
    Ok(())
}

#[command]
pub async fn settings_prefix(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let prefix = match args.next() {
        Some(p) => p,
        None => return Ok(()),
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Prefix": prefix}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(format!(
            "The bot prefix has been successfully changed to {}",
            prefix
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    ctx.config.prefixes.insert(guild_id, prefix.to_string());
    Ok(())
}
