use crate::framework::prelude::*;
use crate::models::{
    command::*,
    guild::RoGuild
};
use itertools::Itertools;

pub static CUSTOMBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["modify", "m"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static CUSTOMBINDS_MODIFY_COMMAND: Command = Command {
    fun: custombinds_modify,
    options: &CUSTOMBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn custombinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
    };

    let bind_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    if !guild.custombinds.iter().any(|c| c.id == bind_id) {
        return Ok(())
    }

    if field.eq_ignore_ascii_case("code") {
        modify_code(ctx, msg, &guild, bind_id, args).await?;
    } else if field.eq_ignore_ascii_case("prefix") {
        modify_prefix(ctx, &guild, bind_id, args.next()).await?;
    } else if field.eq_ignore_ascii_case("priority") {
        modify_priority(ctx, &guild, bind_id, args.next()).await?;
    } else if field.eq_ignore_ascii_case("roles-add") {
        add_roles(ctx, &guild, bind_id, args).await?;
    } else if field.eq_ignore_ascii_case("roles-remove") {
        remove_roles(ctx, &guild, bind_id, args).await?;
    } 

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The bind was successfully modified").unwrap()
        .build().unwrap();

    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;

    Ok(())
}

async fn modify_code(ctx: &Context, msg: &Message, guild: &RoGuild, bind_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
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
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Code": code}};
    ctx.database.modify_guild(filter, update).await
}

async fn modify_prefix(ctx: &Context, guild: &RoGuild, bind_id: i64, prefix: Option<&str>) -> Result<(), RoError> {
    let prefix = match prefix {
        Some(s) => s,
        None => return Ok(())
    };
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Prefix": prefix}};
    ctx.database.modify_guild(filter, update).await
}

async fn modify_priority(ctx: &Context, guild: &RoGuild, bind_id: i64, priority: Option<&str>) -> Result<(), RoError> {
    let priority = match priority.map(|p| p.parse::<i64>()) {
        Some(Ok(p)) => p,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Priority": priority}};
    ctx.database.modify_guild(filter, update).await
}

async fn add_roles(ctx: &Context, guild: &RoGuild, bind_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$push": {"CustomBinds.$.DiscordRoles": {"$each": role_ids}}};
    ctx.database.modify_guild(filter, update).await
}

async fn remove_roles(ctx: &Context, guild: &RoGuild, bind_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$pullAll": {"CustomBinds.$.DiscordRoles": role_ids}};
    ctx.database.modify_guild(filter, update).await
}