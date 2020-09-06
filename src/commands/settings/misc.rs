use crate::framework::prelude::*;
use crate::models::guild::BlacklistActionType;

pub static BLACKLIST_ACTION_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["blacklist-action", "bl-action"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static TOGGLE_COMMANDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["commands"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static BLACKLIST_ACTION_COMMAND: Command = Command {
    fun: blacklist_action,
    options: &BLACKLIST_ACTION_OPTIONS
};

pub static TOGGLE_COMMANDS_COMMAND: Command = Command {
    fun: toggle_commands,
    options: &TOGGLE_COMMANDS_OPTIONS
};

#[command]
pub async fn blacklist_action(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    let bl_type = match option.to_lowercase().as_str() {
        "none" => BlacklistActionType::None,
        "kick" => BlacklistActionType::Kick,
        "ban" => BlacklistActionType::Ban,
        _ => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.BlacklistAction": bl_type as u32}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

//TODO: Not sure what's happening here, this is not working
#[command]
pub async fn toggle_commands(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    
    let update = match option.to_lowercase().as_str() {
        "on" => bson::doc! {"$pull": {"Settings.DisabledChannels": msg.channel_id.0}},
        "off" => bson::doc! {"$push": {"Settings.DisabledChannels": msg.channel_id.0}},
        _ => return Ok(())
    };
    let filter = bson::doc! {"_id": guild.id};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}