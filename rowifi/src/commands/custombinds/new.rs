use itertools::Itertools;
use mongodb::bson::{doc, to_bson};
use roblox::models::id::UserId as RobloxUserId;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::CustomBind,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct CustombindsNewArguments {
    #[arg(help = "The code that makes up the bind", rest)]
    pub code: String,
}

pub async fn custombinds_new(ctx: CommandContext, args: CustombindsNewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let code = args.code;

    let user = match ctx.get_linked_user(ctx.author.id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Bind Addition Failed")
                .unwrap()
                .description("You must be verified to create a custom bind")
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
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx
        .member(ctx.guild_id.unwrap(), ctx.author.id.0)
        .await?
        .unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, r.role.rank as i64))
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

    let prefix = await_reply("Enter the prefix you wish to set for the bind.\nEnter `N/A` if you would not like to set a prefix", &ctx).await?;
    let priority = match await_reply("Enter the priority you wish to set for the bind.", &ctx)
        .await?
        .parse::<i64>()
    {
        Ok(p) => p,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Bind Addition Failed")
                .unwrap()
                .description("Expected priority to be a number")
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

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId(role_id)) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let mut binds = guild.custombinds.iter().map(|c| c.id).collect_vec();
    binds.sort_unstable();
    let id = binds.last().unwrap_or(&0) + 1;
    let bind = CustomBind {
        id,
        code: code.clone(),
        prefix: Some(prefix),
        priority,
        command,
        discord_roles,
        template: None,
    };
    let bind_bson = to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"CustomBinds": bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", bind.id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Code: {}\nPrefix: {}\nPriority: {}\nDiscord Roles: {}",
        bind.code,
        bind.prefix.unwrap(),
        bind.priority,
        roles_str
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .unwrap()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
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
        .description("Custom Bind Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
