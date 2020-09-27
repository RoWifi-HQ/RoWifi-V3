use crate::framework::prelude::*;
use crate::models::blacklist::*;
use itertools::Itertools;

pub static BLACKLISTS_GROUP_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["group"],
    desc: Some("Command to add a group blacklist"),
    usage: Some("blacklist group <Group Id> <Reason>"),
    examples: &["blacklist group 3108077 Test", "bl group 5581309 Not Allowed"],
    required_permissions: Permissions::empty(),
    min_args: 2,
    hidden: false,
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
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let group_id = match args.next() {
        Some(g) => match g.parse::<i64>() {
            Ok(g) => g,
            Err(_) => return Err(CommandError::ParseArgument(g.into(), "Group Id".into(), "Number".into()).into())
        }
        None => return Ok(())
    };
    
    let mut reason = args.join(" ");
    if reason.is_empty() {
        reason = "N/A".into();
    }
    let blacklist = Blacklist {id: group_id.to_string(), reason, blacklist_type: BlacklistType::Group(group_id)};
    let blacklist_bson = bson::to_bson(&blacklist)?;
    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"Blacklists": blacklist_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);

    let embed = EmbedBuilder::new().default_data().title("Blacklist Addition Successful").unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .color(Color::DarkGreen as u32).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;

    let log_embed = EmbedBuilder::new().default_data().title(format!("Action by {}", msg.author.name)).unwrap()
        .description("Blacklist Addition").unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}