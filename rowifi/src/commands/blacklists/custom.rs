use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::{
    blacklist::{Blacklist, BlacklistType},
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

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
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let code = args.code;
    if code.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Custom Blacklist Addition Failed")
            .description("No code was found. Please try again")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }
    let user = match ctx.get_linked_user(ctx.author.id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Blacklist Addition Failed")
                .description("You must be verified to create a custom blacklist")
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx.member(guild_id, ctx.author.id.0).await?.unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(s).await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(res).await?;
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
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .color(Color::DarkGreen as u32)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Blacklist Addition")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
