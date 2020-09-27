use crate::framework::prelude::*;

pub static SETTINGS_VERIFICATION_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["verification", "unverfied"],
    desc: Some("Command to change the verification/unverified role"),
    usage: Some("settings verification @Verification"),
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static SETTINGS_VERIFIED_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["verified"],
    desc: Some("Command to change the verified role"),
    usage: Some("settings verified @Verified"),
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static SETTINGS_VERIFICATION_COMMAND: Command = Command {
    fun: settings_verification,
    options: &SETTINGS_VERIFICATION_OPTIONS
};

pub static SETTINGS_VERIFIED_COMMAND: Command = Command {
    fun: settings_verified,
    options: &SETTINGS_VERIFIED_OPTIONS
};

#[command]
pub async fn settings_verification(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let verification_role = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Verification Role".into(), "Discord Role".into()).into())
        }
        None => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"VerificationRole": verification_role}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .description(format!("The Verification Role was successfully set to <@&{}>", verification_role)).unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description(format!("Settings Modification: Verification Role set to <@&{}>", verification_role)).unwrap()
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

#[command]
pub async fn settings_verified(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let verified_role = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Verified Role".into(), "Discord Role".into()).into())
        }
        None => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"VerifiedRole": verified_role}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .description(format!("The Verified Role was successfully set to <@&{}>", verified_role)).unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description(format!("Settings Modification: Verified Role set to <@&{}>", verified_role)).unwrap()
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}