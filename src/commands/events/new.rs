use rowifi_framework::prelude::*;
use bson::oid::ObjectId;
use rowifi_models::{events::EventLog, guild::GuildType};
use twilight_mention::Mention;

pub static EVENT_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Trainer,
    bucket: None,
    names: &["new"],
    desc: Some("Command to register a new event"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_NEW_COMMAND: Command = Command {
    fun: event_new,
    options: &EVENT_NEW_OPTIONS,
};

#[command]
pub async fn event_new(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
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

    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Event Addition Failed")
                .unwrap()
                .description("This command may only be used by verified users")
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
    };

    let event_type_id = match await_reply("Enter the id of the type of event", ctx, msg)
        .await?
        .parse::<i64>()
    {
        Ok(i) => i,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Event Addition Failed")
                .unwrap()
                .description("The event id has to be a number")
                .unwrap()
                .build()
                .unwrap();
            ctx.http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    let event_type = match guild.event_types.iter().find(|e| e.id == event_type_id) {
        Some(e) => e,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Event Addition Failed")
                .unwrap()
                .description(format!(
                    "An event type with id {} does not exist",
                    event_type_id
                ))
                .unwrap()
                .build()
                .unwrap();
            ctx.http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let attendees_str = await_reply("Enter the list of attendees in this event", ctx, msg).await?;
    let mut attendees = Vec::new();
    for attendee in attendees_str.split(|c| c == ' ' || c == ',') {
        if let Ok(Some(roblox_id)) = ctx.roblox.get_id_from_username(&attendee).await {
            attendees.push(roblox_id);
        }
    }

    if attendees.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Event Addition Failed")
            .unwrap()
            .description("The number of valid attendees was found to be zero")
            .unwrap()
            .build()
            .unwrap();
        ctx.http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let notes_raw = await_reply("Would you like to add any notes to this event log? Say N/A if you would like to not add any notes", ctx, msg).await?;
    let notes = if notes_raw.eq_ignore_ascii_case("N/A") {
        None
    } else {
        Some(notes_raw)
    };

    let event_id = ObjectId::new();
    let guild_id = guild_id.0 as i64;

    let new_event = EventLog {
        id: event_id,
        guild_id,
        event_type: event_type_id,
        guild_event_id: guild.event_counter + 1,
        host_id: user.roblox_id,
        attendees,
        timestamp: bson::DateTime {
            0: chrono::Utc::now(),
        },
        notes,
    };

    ctx.database.add_event(guild_id, &new_event).await?;

    let value = format!(
        "Host: {}\nType: {}\nAttendees: {}",
        msg.author.id.mention(),
        event_type.name,
        new_event.attendees.len()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Event Addition Successful")
        .unwrap()
        .field(
            EmbedFieldBuilder::new(format!("Event Id: {}", guild.event_counter + 1), value)
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
