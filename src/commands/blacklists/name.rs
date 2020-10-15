use crate::framework::prelude::*;
use crate::models::blacklist::*;
use itertools::Itertools;

pub static BLACKLISTS_NAME_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["name"],
    desc: Some("Command to add a user blacklist"),
    usage: Some("blacklist name <Username> <Reason>"),
    examples: &[
        "blacklist name AsianIntel Owner",
        "bl name Zanance Co-Founder",
    ],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static BLACKLISTS_NAME_COMMAND: Command = Command {
    fun: blacklists_name,
    options: &BLACKLISTS_NAME_OPTIONS,
};

#[command]
pub async fn blacklists_name(
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

    let username = match args.next() {
        Some(u) => u.to_owned(),
        None => return Ok(()),
    };
    let user_id = match ctx.roblox.get_id_from_username(&username).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Blacklist Addition Failed")
                .unwrap()
                .description(format!(
                    "There was no user found with username {}",
                    username
                ))
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let mut reason = args.join(" ");
    if reason.is_empty() {
        reason = "N/A".into();
    }

    let blacklist = Blacklist {
        id: user_id.to_string(),
        reason,
        blacklist_type: BlacklistType::Name(user_id.to_string()),
    };
    let blacklist_bson = bson::to_bson(&blacklist)?;
    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"Blacklists": blacklist_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Blacklist Addition Successful")
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .color(Color::DarkGreen as u32)
        .unwrap()
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Blacklist Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
