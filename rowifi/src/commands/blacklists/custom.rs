use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::{
    blacklist::{Blacklist, BlacklistType},
    rolang::{RoCommand, RoCommandUser},
};

#[derive(FromArgs)]
pub struct BlacklistCustomArguments {
    #[arg(help = "Code to use in the blacklist", rest)]
    pub code: String,
}

pub async fn blacklist_custom(
    ctx: CommandContext,
    args: BlacklistCustomArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let code = args.code;
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    let user = match ctx.bot.database.get_user(ctx.author.id.0).await? {
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    let member = ctx.member(guild_id, ctx.author.id.0).await?.unwrap();
    let ranks = ctx.bot.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.bot.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &username,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .content(s)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .content(res)
            .unwrap()
            .await?;
        return Ok(());
    }
    let reason = await_reply("Enter the reason of this blacklist.", &ctx).await?;

    let blacklist = Blacklist {
        id: code.to_string(),
        reason,
        blacklist_type: BlacklistType::Custom(command),
    };
    let blacklist_bson = to_bson(&blacklist)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Blacklists": blacklist_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Blacklist Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
