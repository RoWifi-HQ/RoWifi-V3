use crate::framework::prelude::*;

pub static UPDATE_JOIN_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["update-on-join", "uoj"],
    desc: Some("Command to enable/disable updating a member when they join the server"),
    usage: Some("settings update-on-join <on/off/enable/disable>"),
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static UPDATE_VERIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["update-on-verify", "uov"],
    desc: Some("Command to enable/disable automatically updating a member just after they verify"),
    usage: Some("settings update-on-verify <on/off/enable/disable>"),
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static UPDATE_JOIN_COMMAND: Command = Command {
    fun: update_on_join,
    options: &UPDATE_JOIN_OPTIONS
};

pub static UPDATE_VERIFY_COMMAND: Command = Command {
    fun: update_on_verify,
    options: &UPDATE_VERIFY_OPTIONS
};

#[command]
pub async fn update_on_join(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let option_str = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    let (option, desc) = match option_str.to_lowercase().as_str() {
        "on" | "enable" => (true, "Update on Join has succesfully been enabled"),
        "off" | "disable" => (false, "Update on Join has successfully been disabled"),
        _ => return Err(CommandError::ParseArgument(option_str, "Update On Join".into(), "`on`, `off`, `enable`, `disable`".into()).into())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.UpdateOnJoin": option}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .description(desc).unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description(format!("Settings Modification: Update On Join - {} -> {}", guild.settings.update_on_join, option)).unwrap()
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

#[command]
pub async fn update_on_verify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let option_str = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    let (option, desc) = match option_str.to_lowercase().as_str() {
        "on" | "enable" => (true, "Update on Verify has succesfully been enabled"),
        "off" | "disable" => (false, "Update on Verify has successfully been disabled"),
        _ => return Err(CommandError::ParseArgument(option_str, "Update On Verify".into(), "`on`, `off`, `enable`, `disable`".into()).into())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.UpdateOnVerify": option}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .description(desc).unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description(format!("Settings Modification: Update On Verify - {} -> {}", guild.settings.update_on_verify, option)).unwrap()
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}