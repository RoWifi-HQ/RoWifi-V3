use crate::framework::prelude::*;

pub static UPDATE_JOIN_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update-on-join", "uoj"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static UPDATE_VERIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update-on-verify", "uov"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
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
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    let option = match option.to_lowercase().as_str() {
        "on" => true,
        "off" => false,
        _ => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.UpdateOnJoin": option}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[command]
pub async fn update_on_verify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => return Ok(())
    };
    let option = match option.to_lowercase().as_str() {
        "on" => true,
        "off" => false,
        _ => return Ok(())
    };

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$set": {"Settings.UpdateOnVerify": option}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Settings Modification Successful").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}