use crate::framework::prelude::*;
use crate::models::{events::EventType, guild::GuildType};
use itertools::Itertools;

pub static EVENT_TYPE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Trainer,
    bucket: None,
    names: &["types", "type"],
    desc: Some("Command to view the created event types"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[&EVENT_TYPE_NEW_COMMAND, &EVENT_TYPE_MODIFY_COMMAND],
    group: None,
};

pub static EVENT_TYPE_COMMAND: Command = Command {
    fun: event_type,
    options: &EVENT_TYPE_OPTIONS,
};

pub static EVENT_TYPE_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to create a new event type"),
    usage: None,
    examples: &[],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_TYPE_NEW_COMMAND: Command = Command {
    fun: event_type_new,
    options: &EVENT_TYPE_NEW_OPTIONS,
};

pub static EVENT_TYPE_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify"],
    desc: Some("Command to modify an existing event type"),
    usage: None,
    examples: &[],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_TYPE_MODIFY_COMMAND: Command = Command {
    fun: event_type_modify,
    options: &EVENT_TYPE_MODIFY_OPTIONS,
};

#[command]
pub async fn event_type(_ctx: &Context, _msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    //Check for beta tier
    Ok(())
}

#[command]
pub async fn event_type_new(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommandError::NoRoGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Command Failed")
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

    let event_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => {
                return Err(CommandError::ParseArgument(
                    a.into(),
                    "Event ID".into(),
                    "Number".into(),
                )
                .into())
            }
        },
        None => return Ok(()),
    };

    if guild.event_types.iter().any(|e| e.id == event_id) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Event Type Addition Failed").unwrap()
            .description(format!("An event type with id {} already exists. To modify an event type, use `events type modify`", event_id)).unwrap()
            .build().unwrap();
        ctx.http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let event_name = args.join(" ");

    let event_type = EventType {
        id: event_id,
        name: event_name.to_string(),
        xp: 0,
    };
    let event_bson = bson::to_bson(&event_type)?;

    let filter = bson::doc! {"_id": guild_id.0};
    let update = bson::doc! {"$push": {"EventTypes": event_bson}};

    ctx.database.modify_guild(filter, update).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Event Type Addition Successful")
        .unwrap()
        .field(
            EmbedFieldBuilder::new(
                format!("Id: {}", event_type.id),
                format!("Name: {}", event_type.name),
            )
            .unwrap(),
        )
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
pub async fn event_type_modify(
    _ctx: &Context,
    _msg: &Message,
    _args: Arguments<'fut>,
) -> CommandResult {
    //Check for beta tier
    Ok(())
}