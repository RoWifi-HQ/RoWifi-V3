use crate::framework::prelude::*;
use crate::models::{command::*, guild::RoGuild};
use itertools::Itertools;

pub static CUSTOMBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify", "m"],
    desc: Some("Command to modify a custombind"),
    usage: Some("custombinds modify <Field> <Bind Id> [Params...]`\n`Field`: `code`, `priority`, `prefix`, `roles-add`, `roles-remove"),
    examples: &["custombinds modify code 1 HasRank(3108077, 255)", "cb modify priority 1 23", "custombinds m prefix 2 N/A"],
    required_permissions: Permissions::empty(),
    min_args: 3,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static CUSTOMBINDS_MODIFY_COMMAND: Command = Command {
    fun: custombinds_modify,
    options: &CUSTOMBINDS_MODIFY_OPTIONS,
};

#[command]
pub async fn custombinds_modify(
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

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(()),
    };

    let bind_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => {
                return Err(CommandError::ParseArgument(
                    a.into(),
                    "Bind ID".into(),
                    "Number".into(),
                )
                .into())
            }
        },
        None => return Ok(()),
    };

    let bind = match guild.custombinds.iter().find(|c| c.id == bind_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Bind Modification Failed")
                .unwrap()
                .description(format!("There was no bind found with id {}", bind_id))
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap();
            return Ok(());
        }
    };

    let name = format!("Id: {}", bind_id);
    let desc = if field.eq_ignore_ascii_case("code") {
        let new_code = match modify_code(ctx, msg, &guild, bind_id, args).await? {
            Some(n) => n,
            None => return Ok(()),
        };
        format!("`New Code`: {}", new_code)
    } else if field.eq_ignore_ascii_case("prefix") {
        let new_prefix = modify_prefix(ctx, &guild, bind_id, args.next()).await?;
        format!("`Prefix`: {} -> {}", bind.prefix, new_prefix)
    } else if field.eq_ignore_ascii_case("priority") {
        let new_priority = modify_priority(ctx, &guild, bind_id, args.next()).await?;
        format!("`Priority`: {} -> {}", bind.priority, new_priority)
    } else if field.eq_ignore_ascii_case("roles-add") {
        let role_ids = add_roles(ctx, &guild, bind_id, args).await?;
        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        format!("Added Roles: {}", modification)
    } else if field.eq_ignore_ascii_case("roles-remove") {
        let role_ids = remove_roles(ctx, &guild, bind_id, args).await?;
        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        format!("Removed Roles: {}", modification)
    } else {
        return Err(CommandError::ParseArgument(
            field,
            "Field".into(),
            "`prefix`, `priority`, `code`, `roles-add`, `roles-remove`".into(),
        )
        .into());
    };

    let e = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Success!")
        .unwrap()
        .description("The bind was successfully modified")
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Custom Bind Modification")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

async fn modify_code(
    ctx: &Context,
    msg: &Message,
    guild: &RoGuild,
    bind_id: i64,
    mut args: Arguments<'_>,
) -> Result<Option<String>, RoError> {
    let code = args.join(" ");
    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Bind Modification Failed")
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
            return Ok(None);
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
            return Ok(None);
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .content(res)
            .unwrap()
            .await;
        return Ok(None);
    }
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Code": code.clone()}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(Some(code))
}

async fn modify_prefix(
    ctx: &Context,
    guild: &RoGuild,
    bind_id: i64,
    prefix: Option<&str>,
) -> Result<String, RoError> {
    let prefix = prefix.unwrap();
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Prefix": prefix}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(prefix.to_string())
}

async fn modify_priority(
    ctx: &Context,
    guild: &RoGuild,
    bind_id: i64,
    priority: Option<&str>,
) -> Result<i64, RoError> {
    let priority = match priority.unwrap().parse::<i64>() {
        Ok(p) => p,
        Err(_) => {
            return Err(CommandError::ParseArgument(
                priority.unwrap().into(),
                "Priority".into(),
                "Number".into(),
            )
            .into())
        }
    };
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$set": {"CustomBinds.$.Priority": priority}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn add_roles(
    ctx: &Context,
    guild: &RoGuild,
    bind_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$push": {"CustomBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &Context,
    guild: &RoGuild,
    bind_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = bson::doc! {"$pullAll": {"CustomBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}
