use crate::framework::prelude::*;

pub static SETTINGS_VERIFICATION_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["verification"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static SETTINGS_VERIFIED_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["verified"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
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
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let verification_role = match args.next().map(|r| parse_role(r)) {
        Some(Some(s)) => s,
        Some(None) => return Ok(()),
        None => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"VerificationRole": verification_role}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[command]
pub async fn settings_verified(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let verified_role = match args.next().map(|r| parse_role(r)) {
        Some(Some(s)) => s,
        Some(None) => return Ok(()),
        None => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"VerifiedRole": verified_role}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}