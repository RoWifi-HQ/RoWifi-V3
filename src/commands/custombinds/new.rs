use rowifi_framework::prelude::*;
use itertools::Itertools;
use rowifi_models::{
    bind::CustomBind,
    rolang::{RoCommand, RoCommandUser},
};

pub static CUSTOMBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to add a custombind"),
    usage: Some("custombinds new <Code>"),
    examples: &["custombinds new HasRank(3108077, 255) and GetRank(3455445) >= 120"],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static CUSTOMBINDS_NEW_COMMAND: Command = Command {
    fun: custombinds_new,
    options: &CUSTOMBINDS_NEW_OPTIONS,
};

#[command]
pub async fn custombinds_new(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let code = args.as_str();

    let user = match ctx.database.get_user(msg.author.id.0).await? {
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    let member = ctx.member(guild_id, msg.author.id.0).await?.unwrap();
    let ranks = ctx.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &username,
    };
    let command = match RoCommand::new(code) {
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

    let prefix = await_reply("Enter the prefix you wish to set for the bind.\nEnter `N/A` if you would not like to set a prefix", ctx, msg).await?;
    let priority = match await_reply("Enter the priority you wish to set for the bind.", ctx, msg)
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let server_roles = ctx.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", ctx, msg).await?;
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
        code: code.to_owned(),
        prefix,
        priority,
        command,
        discord_roles,
    };
    let bind_bson = bson::to_bson(&bind)?;
    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"CustomBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", bind.id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Code: {}\nPrefix: {}\nPriority: {}\nDiscord Roles: {}",
        bind.code, bind.prefix, bind.priority, roles_str
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
        .description("Custom Bind Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
