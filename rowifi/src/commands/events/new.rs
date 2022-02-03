use rowifi_database::{encrypt_bytes, postgres::Row};
use rowifi_framework::prelude::*;
use rowifi_models::{
    events::{EventLog, EventType},
    guild::GuildType,
    id::{EventId, UserId},
};
use std::time::Duration;

pub async fn events_new(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    if guild.kind != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let roblox_id = match ctx
        .bot
        .database
        .get_linked_user(UserId(ctx.author.id), guild_id)
        .await?
    {
        Some(r) => r.roblox_id,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Event Addition Failed")
                .description("You need to be verified to use this command")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id)],
        )
        .await?;

    let mut options = Vec::new();
    for event_type in &event_types {
        if !event_type.disabled {
            options.push(SelectMenuOption {
                default: false,
                description: None,
                emoji: None,
                label: event_type.name.clone(),
                value: event_type.event_type_guild_id.to_string(),
            });
        }
    }

    if options.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Event Addition Failed")
            .description("There are no event types or all event types are disabled")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut select_menu = SelectMenu {
        custom_id: "event-new-select".into(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options,
        placeholder: None,
    };

    let message = ctx
        .respond()
        .content("Select an event type")?
        .components(&[
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(select_menu.clone())],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some("event-new-cancel".into()),
                    disabled: false,
                    emoji: None,
                    label: Some("Cancel".into()),
                    style: ButtonStyle::Danger,
                    url: None,
                })],
            }),
        ])?
        .exec()
        .await?
        .model()
        .await?;

    select_menu.disabled = true;

    let message_id = message.id;
    let author_id = ctx.author.id;
    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);
    let mut event_guild_id = None;
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    ctx.bot
                        .http
                        .interaction(ctx.bot.application_id)
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(vec![Component::ActionRow(ActionRow {
                                    components: vec![Component::SelectMenu(select_menu.clone())],
                                })]),
                                embeds: None,
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await?;
                    if message_component.data.custom_id == "event-new-cancel" {
                        ctx.bot
                            .http
                            .interaction(ctx.bot.application_id)
                            .create_followup_message(&message_component.token)
                            .content("Command has been cancelled")?
                            .exec()
                            .await?;
                    } else if message_component.data.custom_id == "event-new-select" {
                        event_guild_id = Some(message_component.data.values[0].clone());
                    }
                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction(ctx.bot.application_id)
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        &InteractionResponse::DeferredUpdateMessage,
                    )
                    .exec()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .interaction(ctx.bot.application_id)
                    .create_followup_message(&message_component.token)
                    .ephemeral(true)
                    .content("This button is only interactable by the original command invoker")?
                    .exec()
                    .await;
            }
        }
    }

    let event_guild_id = match event_guild_id {
        Some(e) => e.parse::<i32>().unwrap(),
        None => return Ok(()),
    };

    let event_type = event_types
        .iter()
        .find(|e| e.event_type_guild_id == event_guild_id)
        .unwrap();

    let attendees_str = await_reply(
        "Enter the usernames of Roblox Users who attended this event",
        &ctx,
    )
    .await?;
    let mut attendees = Vec::new();
    for attendee in attendees_str.split(|c| c == ' ' || c == ',') {
        if let Ok(Some(roblox_id)) = ctx.bot.roblox.get_user_from_username(attendee).await {
            attendees.push(roblox_id);
        }
    }

    if attendees.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Event Addition Failed")
            .description("The number of valid attendees was found to be zero")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }
    let timestamp = chrono::Utc::now();

    let notes_raw = await_reply("Would you like to add any notes to this event log? Say N/A if you would like to not add any notes", &ctx).await?;
    let notes = if notes_raw.eq_ignore_ascii_case("N/A") {
        None
    } else {
        Some(encrypt_bytes(
            notes_raw.as_bytes(),
            &ctx.bot.database.cipher,
            guild_id.get(),
            roblox_id as u64,
            timestamp.timestamp() as u64,
        ))
    };

    let new_event = EventLog {
        event_id: EventId::default(),
        guild_id,
        event_type: event_type.event_type_guild_id,
        guild_event_id: 0,
        host_id: roblox_id,
        attendees: attendees.iter().map(|a| a.id.0 as i64).collect(),
        timestamp,
        notes,
    };

    let row = ctx.bot.database.query_one::<Row>(
        r#"INSERT INTO events(guild_id, event_type, guild_event_id, host_id, timestamp, attendees, notes)
        VALUES($1, $2, (SELECT COALESCE(max(guild_event_id) + 1, 1) FROM events WHERE guild_id = $1), $3, $4, $5, $6)
        RETURNING guild_event_id"#,
        &[&new_event.guild_id, &new_event.event_type, &new_event.host_id, &new_event.timestamp, &new_event.attendees, &new_event.notes]
    ).await?;

    let value = format!(
        "Host: <@{}>\nType: {}\nAttendees: {}",
        ctx.author.id.get(),
        event_type.name,
        new_event.attendees.len()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Event Addition Successful")
        .field(EmbedFieldBuilder::new(
            format!("Event Id: {}", row.get::<'_, _, i64>("guild_event_id")),
            value,
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}
