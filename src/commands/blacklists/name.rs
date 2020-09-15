use crate::framework::prelude::*;
use crate::models::blacklist::*;
use itertools::Itertools;

pub static BLACKLISTS_NAME_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["name"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static BLACKLISTS_NAME_COMMAND: Command = Command {
    fun: blacklists_name,
    options: &BLACKLISTS_NAME_OPTIONS
};

#[command]
pub async fn blacklists_name(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let username = match args.next() {
        Some(u) => u.to_owned(),
        None => return Ok(())
    };
    let user_id = match ctx.roblox.get_id_from_username(&username).await? {
        Some(u) => u,
        None => return Ok(())
    };
    //Check if you put the username or user id in blacklist type name, not that it matters but still
    let reason = args.join(" ");
    let blacklist = Blacklist {id: user_id.to_string(), reason, blacklist_type: BlacklistType::Name(user_id.to_string())};
    let blacklist_bson = bson::to_bson(&blacklist)?;
    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"Blacklists": blacklist_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new().default_data().title("Blacklist Addition Successful").unwrap()
        .color(Color::DarkGreen as u32).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;
    Ok(())
}