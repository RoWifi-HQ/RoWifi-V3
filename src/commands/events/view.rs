use crate::framework::prelude::*;
use crate::models::guild::GuildType;
use bson::doc;

pub static EVENT_ATTENDEE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["attendee"],
    desc: Some("Command to view the last 12 events attended by the given user"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_ATTENDEE_COMMAND: Command = Command {
    fun: event_attendee,
    options: &EVENT_ATTENDEE_OPTIONS,
};

pub static EVENT_HOST_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["host"],
    desc: Some("Command to view the last 12 events hosted by the given user"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_HOST_COMMAND: Command = Command {
    fun: event_host,
    options: &EVENT_HOST_OPTIONS,
};

#[command]
pub async fn event_attendee(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or(CommandError::NoRoGuild)?;

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

    let user_id = match args.next().and_then(|x| x.parse::<i64>().ok()) {
        Some(s) => s,
        None => {
            let user = ctx.database.get_user(msg.author.id.0).await?;
            match user {
                Some(u) => u.roblox_id as i64,
                None => {
                        let embed = EmbedBuilder::new()
                        .default_data()
                        .title("Event Viewing Failed")
                        .unwrap()
                        .description("You must be verified to use this command on yourselves")
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
                        .await;
                        return Ok(());
                }
            }
        }
    };

    let pipeline = vec![
        doc! {"$match": {"GuildId": guild_id.0}},
        doc! {"$sort": {"Timestamp": -1}},
        doc! {"$unwind": "$Attendees"},
        doc! {"$match": {"Attendees": user_id}},
        doc! {"$limit": 12},
        doc! {"$unset": "Attendees"},
    ];
    let events = ctx.database.get_events(pipeline).await?;

    let mut embed = EmbedBuilder::new().default_data()
        .title("Events").unwrap();
    for event in events {
        let name = format!("Id: {}", event.guild_event_id);

        let event_type = guild.event_types.iter().find(|e| e.id == event.event_type).unwrap();
        let host = ctx.roblox.get_username_from_id(event.host_id).await?;
        let desc = format!("Event Type: {}\nHost: {}\nTimestamp:{}", event_type.name, host, event.timestamp.to_rfc3339());

        embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline());
    }

    let embed = embed.build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[command]
pub async fn event_host(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or(CommandError::NoRoGuild)?;

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

    let user_id = match args.next().and_then(|x| x.parse::<i64>().ok()) {
        Some(s) => s,
        None => {
            let user = ctx.database.get_user(msg.author.id.0).await?;
            match user {
                Some(u) => u.roblox_id as i64,
                None => {
                        let embed = EmbedBuilder::new()
                        .default_data()
                        .title("Event Viewing Failed")
                        .unwrap()
                        .description("You must be verified to use this command on yourselves")
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
                        .await;
                        return Ok(());
                }
            }
        }
    };

    let pipeline = vec![
        doc! {"$match": {"GuildId": guild_id.0}},
        doc! {"$match": {"HostId": user_id}},
        doc! {"$sort": {"Timestamp": -1}},
        doc! {"$limit": 12}
    ];
    let events = ctx.database.get_events(pipeline).await?;

    let mut embed = EmbedBuilder::new().default_data()
        .title("Events").unwrap();
    for event in events {
        let name = format!("Id: {}", event.guild_event_id);

        let event_type = guild.event_types.iter().find(|e| e.id == event.event_type).unwrap();
        let host = ctx.roblox.get_username_from_id(event.host_id).await?;
        let desc = format!("Event Type: {}\nHost: {}\nTimestamp:{}\nAttendees: {}", event_type.name, host, event.timestamp.to_rfc3339(), event.attendees.len());

        embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline());
    }

    let embed = embed.build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;

    Ok(())
}