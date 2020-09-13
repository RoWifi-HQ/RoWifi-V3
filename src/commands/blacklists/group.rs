use crate::framework::prelude::*;
use crate::models::blacklist::*;
use itertools::Itertools;

pub static BLACKLISTS_GROUP_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["group"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static BLACKLISTS_GROUP_COMMAND: Command = Command {
    fun: blacklists_group,
    options: &BLACKLISTS_GROUP_OPTIONS
};

#[command]
pub async fn blacklists_group(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    //Check if you put the username or user id in blacklist type name, not that it matters but still
    let reason = args.join(" ");
    let blacklist = Blacklist {id: group_id.to_string(), reason, blacklist_type: BlacklistType::Group(group_id)};
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