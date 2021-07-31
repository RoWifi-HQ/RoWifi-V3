use chacha20poly1305::{aead::Aead, Nonce};
use mongodb::bson::{oid::ObjectId, DateTime};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rowifi_framework::prelude::*;
use rowifi_models::{events::EventLog, guild::GuildType};
use std::time::Duration;
use twilight_mention::Mention;

pub async fn events_new(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let user = match ctx.get_linked_user(ctx.author.id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Event Addition Failed")
                .description("This command may only be used by verified users")
                .color(Color::Red as u32)
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let mut options = Vec::new();
    for event_type in &guild.event_types {
        options.push(SelectMenuOption {
            default: false,
            description: None,
            emoji: None,
            label: event_type.name.clone(),
            value: event_type.id.to_string(),
        });
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
        .bot
        .http
        .create_message(ctx.channel_id)
        .content("Select an event type")
        .unwrap()
        .components(vec![
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
        ])
        .unwrap()
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
    let mut event_type_id = None;
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(vec![Component::ActionRow(ActionRow {
                                    components: vec![Component::SelectMenu(select_menu.clone())],
                                })]),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .await?;
                    if message_component.data.custom_id == "event-new-cancel" {
                        ctx.bot
                            .http
                            .create_followup_message(&message_component.token)
                            .unwrap()
                            .content("Command has been cancelled")
                            .await?;
                    } else if message_component.data.custom_id == "event-new-select" {
                        event_type_id = Some(message_component.data.values[0].clone());
                    }
                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        InteractionResponse::DeferredUpdateMessage,
                    )
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This button is only interactable by the original command invoker")
                    .await;
            }
        }
    }

    let event_type_id = match event_type_id {
        Some(e) => e.parse().unwrap(),
        None => {
            ctx.bot
                .http
                .update_message(message.channel_id, message_id)
                .components(None)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let event_type = guild
        .event_types
        .iter()
        .find(|e| e.id == event_type_id)
        .unwrap();

    let attendees_str = await_reply("Enter the list of attendees in this event", &ctx).await?;
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embeds(vec![embed])
            .unwrap()
            .await?;
        return Ok(());
    }

    let notes_raw = await_reply("Would you like to add any notes to this event log? Say N/A if you would like to not add any notes", &ctx).await?;
    let notes = if notes_raw.eq_ignore_ascii_case("N/A") {
        None
    } else {
        let nonce_str = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(12)
            .map(char::from)
            .collect::<String>();
        let nonce = Nonce::from_slice(nonce_str.as_bytes());
        let ciphertext = ctx.bot.cipher.encrypt(nonce, notes_raw.as_bytes()).unwrap();
        let notes = base64::encode(ciphertext);
        Some((nonce_str, notes))
    };

    let event_id = ObjectId::new();
    let guild_id = guild_id.0;

    let new_event = EventLog {
        id: event_id,
        guild_id: guild_id as i64,
        event_type: event_type_id,
        guild_event_id: guild.event_counter + 1,
        host_id: user.roblox_id,
        attendees: attendees.iter().map(|a| a.id.0 as i64).collect(),
        timestamp: DateTime::from(chrono::Utc::now()),
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
        .title("Event Addition Successful")
        .field(EmbedFieldBuilder::new(
            format!("Event Id: {}", guild.event_counter + 1),
            value,
        ))
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embeds(vec![embed])
        .unwrap()
        .await?;
    Ok(())
}
