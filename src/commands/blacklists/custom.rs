use crate::framework::prelude::*;
use crate::models::{blacklist::*, command::*};
use itertools::Itertools;
use tokio::time::timeout;
use std::time::Duration;
use twilight_model::gateway::payload::MessageCreate;

pub static BLACKLISTS_CUSTOM_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["custom"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static BLACKLISTS_CUSTOM_COMMAND: Command = Command {
    fun: blacklists_custom,
    options: &BLACKLISTS_CUSTOM_OPTIONS
};

#[command]
pub async fn blacklists_custom(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;
    
    let code = args.join(" ");
    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => return Ok(())
    };
    let member = ctx.member(msg.guild_id.unwrap(), msg.author.id.0).await?.unwrap();
    let ranks = ctx.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {user: &user, member, ranks: &ranks, username: &username};
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            let _ = ctx.http.create_message(msg.channel_id).content(s).unwrap().await?;
            return Ok(())
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        let _ = ctx.http.create_message(msg.channel_id).content(res).unwrap().await;
        return Ok(())
    }
    let id = msg.author.id;
    let _ = ctx.http.create_message(msg.channel_id).content("Enter the reason of this blacklist.").unwrap().await;
    let fut = ctx.standby.wait_for_message(msg.channel_id, move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
    let reason = match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => {
            m.content.to_owned()
        },
        _ => {
            let e = EmbedBuilder::new()
                .default_data().color(Color::Red as u32).unwrap()
                .title("Bind Addition Failed").unwrap()
                .description("Command has been cancelled. Please try again.").unwrap()
                .build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }
    };

    let blacklist = Blacklist {id: code, reason, blacklist_type: BlacklistType::Custom(command)};
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