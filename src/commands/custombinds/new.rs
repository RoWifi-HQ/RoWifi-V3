use crate::framework::prelude::*;
use crate::models::{
    command::{RoCommandUser, RoCommand},
    bind::CustomBind
};
use std::time::Duration;
use twilight_model::gateway::payload::MessageCreate;
use twilight_embed_builder::EmbedFieldBuilder;
use tokio::time::timeout;
use itertools::Itertools;

pub static CUSTOMBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["new"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static CUSTOMBINDS_NEW_COMMAND: Command = Command {
    fun: custombinds_new,
    options: &CUSTOMBINDS_NEW_OPTIONS
};

#[command]
pub async fn custombinds_new(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult {
    let embed = EmbedBuilder::new()
                .default_data().color(Color::Red as u32).unwrap()
                .title("Bind Addition Failed").unwrap();
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let code = args.as_str();
    if code.is_empty() {
        return Ok(())
    }

    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => return Ok(())
    };
    let member = ctx.member(guild_id, msg.author.id.0).await?.unwrap();
    let ranks = ctx.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {user: &user, member, ranks: &ranks, username: &username};
    let command = match RoCommand::new(code) {
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

    //Get the prefix, priority & roles
    let id = msg.author.id;
    let _ = ctx.http.create_message(msg.channel_id).content("Enter the prefix you wish to set for the bind.\nEnter `N/A` if you would not like to set a prefix").unwrap().await;
    let fut = ctx.standby.wait_for_message(msg.channel_id, move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
    let prefix = match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => {
            m.content.to_owned()
        },
        _ => {
            let e = embed.description("Command has been cancelled. Please try again.").unwrap().build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }
    };

    let _ = ctx.http.create_message(msg.channel_id).content("Enter the priority you wish to set for the bind.").unwrap().await;
    let fut = ctx.standby.wait_for_message(msg.channel_id, move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
    let priority = match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => {
            match m.content.parse::<i64>() {
                Ok(p) => p,
                Err(_) => {
                    let e = embed.description("Invalid priority found. Please try again.").unwrap().build().unwrap();
                    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
                    return Ok(())
                }
            }
        },
        _ => {
            let e = embed.description("Command has been cancelled. Please try again.").unwrap().build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }
    };

    let _ = ctx.http.create_message(msg.channel_id).content("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.").unwrap().await;
    let server_roles = ctx.cache.roles(guild_id);
    let fut = ctx.standby.wait_for_message(msg.channel_id, move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
    let discord_roles = match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => {
            let mut roles_str = m.content.split_whitespace();
            let mut roles = Vec::new();
            while let Some(r) = roles_str.next() {
                if let Some(role_id) = parse_role(r) {
                    if server_roles.contains(&RoleId(role_id as u64)) {
                        roles.push(role_id as i64);
                    }
                }
            }
            roles
        },
        _ => {
            let e = embed.description("Command has been cancelled. Please try again.").unwrap().build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }
    };

    let mut binds = guild.custombinds.iter().map(|c| c.id).collect_vec();
    binds.sort();
    let id = binds.last().unwrap_or(&0) + 1;
    let bind = CustomBind {id, code: code.to_owned(), prefix, priority, command, discord_roles};
    let bind_bson = bson::to_bson(&bind)?;
    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"CustomBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;
    
    let name = format!("Id: {}", bind.id);
    let roles_str = bind.discord_roles.iter().map(|r| format!("<@&{}> ", r)).collect::<String>();
    let desc = format!("Code: {}\nPrefix: {}\nPriority: {}\nDiscord Roles: {}", bind.code, bind.prefix, bind.priority, roles_str);
    let embed = EmbedBuilder::new().default_data().title("Bind Addition Successful").unwrap()
        .color(Color::DarkGreen as u32).unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;
    Ok(())
}