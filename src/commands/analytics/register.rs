use crate::framework::prelude::*;
use crate::models::guild::GuildType;

pub static ANALYTICS_REGISTER_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["register"],
    desc: Some("Command to register a new group in analytics module"),
    usage: Some("analytics register <Group Id>"),
    examples: &["analytics register 3108077"],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static ANALYTICS_REGISTER_COMMAND: Command = Command {
    fun: analytics_register,
    options: &ANALYTICS_REGISTER_OPTIONS,
};

pub static ANALYTICS_UNREGISTER_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["unregister"],
    desc: Some("Command to unregister an existing group in analytics module"),
    usage: Some("analytics unregister <Group Id>"),
    examples: &["analytics unregister 3108077"],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static ANALYTICS_UNREGISTER_COMMAND: Command = Command {
    fun: analytics_unregister,
    options: &ANALYTICS_UNREGISTER_OPTIONS,
};

#[command]
pub async fn analytics_register(
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

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Group Registration Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
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

    let group_id = match args.next() {
        Some(group_str) => match group_str.parse::<i64>() {
            Ok(g) => g,
            Err(_) => {
                return Err(RoError::Command(CommandError::ParseArgument(
                    group_str.to_string(),
                    "Group Id".into(),
                    "Number".into(),
                )))
            }
        },
        None => return Ok(()),
    };

    if guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration Already Exists")
            .unwrap()
            .color(Color::Red as u32)
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

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"RegisteredGroups": group_id}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Group Registration Successful")
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[command]
pub async fn analytics_unregister(
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

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Group Registration Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
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

    let group_id = match args.next() {
        Some(group_str) => match group_str.parse::<i64>() {
            Ok(g) => g,
            Err(_) => {
                return Err(RoError::Command(CommandError::ParseArgument(
                    group_str.to_string(),
                    "Group Id".into(),
                    "Number".into(),
                )))
            }
        },
        None => return Ok(()),
    };

    if !guild.registered_groups.iter().any(|g| g == &group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Group Registration doesn't exist")
            .unwrap()
            .color(Color::Red as u32)
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

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"RegisteredGroups": group_id}};
    ctx.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Group Unregistration Successful")
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
