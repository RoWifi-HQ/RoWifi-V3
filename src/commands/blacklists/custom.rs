use crate::framework::prelude::*;
use crate::models::{blacklist::*, command::*};
use itertools::Itertools;

pub static BLACKLISTS_CUSTOM_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["custom"],
    desc: Some("Command to add a custom blacklist"),
    usage: Some("blacklists custom <Code>"),
    examples: &["blacklists custom not IsInGroup(3108077)"],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static BLACKLISTS_CUSTOM_COMMAND: Command = Command {
    fun: blacklists_custom,
    options: &BLACKLISTS_CUSTOM_OPTIONS,
};

#[command]
pub async fn blacklists_custom(
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

    let code = args.join(" ");
    if code.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Custom Blacklist Addition Failed")
            .unwrap()
            .description("No code was found. Please try again")
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
    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Blacklist Addition Failed")
                .unwrap()
                .description("You must be verified to create a custom blacklist")
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
    let member = ctx
        .member(msg.guild_id.unwrap(), msg.author.id.0)
        .await?
        .unwrap();
    let ranks = ctx.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        member,
        ranks: &ranks,
        username: &username,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .content(s)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .content(res)
            .unwrap()
            .await;
        return Ok(());
    }
    let reason = await_reply("Enter the reason of this blacklist.", ctx, msg).await?;

    let blacklist = Blacklist {
        id: code,
        reason,
        blacklist_type: BlacklistType::Custom(command),
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
