use chacha20poly1305::{Nonce, aead::Aead};
use mongodb::bson::{oid::ObjectId, DateTime};
use rand::{Rng, thread_rng, distributions::Alphanumeric};
use rowifi_framework::prelude::*;
use rowifi_models::{events::EventLog, guild::GuildType};
use twilight_mention::Mention;

use super::EventArguments;

pub async fn events_new(ctx: CommandContext, _args: EventArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
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
                .title("Event Addition Failed")
                .unwrap()
                .description("This command may only be used by verified users")
                .unwrap()
                .color(Color::Red as u32)
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

    let event_type_id = match await_reply("Enter the id of the type of event", &ctx)
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let attendees_str = await_reply("Enter the list of attendees in this event", &ctx).await?;
    let mut attendees = Vec::new();
    for attendee in attendees_str.split(|c| c == ' ' || c == ',') {
        if let Ok(Some(roblox_id)) = ctx.bot.roblox.get_id_from_username(&attendee).await {
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let notes_raw = await_reply("Would you like to add any notes to this event log? Say N/A if you would like to not add any notes", &ctx).await?;
    let notes = if notes_raw.eq_ignore_ascii_case("N/A") {
        None
    } else {
        let nonce_str = thread_rng().sample_iter(&Alphanumeric).take(12).map(char::from).collect::<String>();
        let nonce = Nonce::from_slice(nonce_str.as_bytes());
        let ciphertext = ctx.bot.cipher.encrypt(nonce, notes_raw.as_bytes()).unwrap();
        let notes = base64::encode(ciphertext);
        Some((nonce_str, notes))
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
        timestamp: DateTime {
            0: chrono::Utc::now(),
        },
        notes,
    };

    ctx.bot.database.add_event(guild_id, &new_event).await?;

    let value = format!(
        "Host: {}\nType: {}\nAttendees: {}",
        ctx.author.id.mention(),
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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
