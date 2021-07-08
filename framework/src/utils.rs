use crate::{
    context::CommandContext,
    error::{CommandError, RoError},
};

use std::{
    cmp::{max, min},
    time::Duration,
};
use tokio::time::timeout;
use tokio_stream::StreamExt;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        component::{
            action_row::ActionRow,
            button::{Button, ButtonStyle},
            Component, ComponentEmoji,
        },
        interaction::Interaction,
    },
    channel::embed::Embed,
    gateway::{event::Event, payload::MessageCreate},
};

pub enum Color {
    Red = 0x00E7_4C3C,
    Blue = 0x0034_98DB,
    DarkGreen = 0x001F_8B4C,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
#[repr(i8)]
pub enum RoLevel {
    Creator = 3,
    Admin = 2,
    Trainer = 1,
    Normal = 0,
}

impl Default for RoLevel {
    fn default() -> Self {
        RoLevel::Normal
    }
}

pub async fn await_reply(question: &str, ctx: &CommandContext) -> Result<String, RoError> {
    let question = format!("{}\nSay `cancel` to cancel this prompt", question);
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .content(question)
        .unwrap()
        .await?;
    let id = ctx.author.id;
    let fut = ctx
        .bot
        .standby
        .wait_for_message(ctx.channel_id, move |event: &MessageCreate| {
            event.author.id == id && !event.content.is_empty()
        });
    match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => Ok(m.content.clone()),
        _ => Err(RoError::Command(CommandError::Timeout)),
    }
}

pub async fn paginate_embed(
    ctx: &CommandContext,
    pages: Vec<Embed>,
    page_count: usize,
) -> Result<(), RoError> {
    let page_count = page_count;
    if page_count <= 1 {
        let _ = ctx
            .bot
            .http
            .create_message(ctx.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .await?;
    } else {
        let m = ctx
            .bot
            .http
            .create_message(ctx.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .component(Component::ActionRow(ActionRow {
                components: vec![
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ComponentEmoji {
                            id: None,
                            name: "⏮️".into(),
                            animated: false,
                        }),
                        label: Some("First Page".into()),
                        custom_id: Some("first-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ComponentEmoji {
                            id: None,
                            name: "◀️".into(),
                            animated: false,
                        }),
                        label: Some("Previous Page".into()),
                        custom_id: Some("previous-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ComponentEmoji {
                            id: None,
                            name: "▶️".into(),
                            animated: false,
                        }),
                        label: Some("Next Page".into()),
                        custom_id: Some("next-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ComponentEmoji {
                            id: None,
                            name: "⏭️".into(),
                            animated: false,
                        }),
                        label: Some("Last Page".into()),
                        custom_id: Some("last-page".into()),
                        url: None,
                        disabled: false,
                    }),
                ],
            }))
            .unwrap()
            .await?;

        //Get some easy named vars
        let channel_id = m.channel_id;
        let message_id = m.id;
        let author_id = ctx.author.id;
        let http = ctx.bot.http.clone();

        let component_interaction = ctx
            .bot
            .standby
            .wait_for_event_stream(move |event: &Event| {
                if let Event::InteractionCreate(interaction) = event {
                    if let Interaction::MessageComponent(message_component) = &interaction.0 {
                        if message_component.message.id == m.id {
                            return true;
                        }
                    }
                }
                false
            })
            .timeout(Duration::from_secs(300));
        tokio::pin!(component_interaction);

        let mut page_pointer: usize = 0;
        while let Some(Ok(event)) = component_interaction.next().await {
            if let Event::InteractionCreate(interaction) = event {
                if let Interaction::MessageComponent(message_component) = interaction.0 {
                    let component_interaction_author = message_component.as_ref().member.as_ref().unwrap().user.as_ref().unwrap().id;
                    if component_interaction_author == author_id {
                        match message_component.data.custom_id.as_str() {
                            "first-page" => {
                                page_pointer = 0;
                            }
                            "previous-page" => {
                                page_pointer = max(page_pointer - 1, 0);
                            }
                            "next-page" => {
                                page_pointer = min(page_pointer + 1, page_count - 1);
                            }
                            "last-page" => {
                                page_pointer = page_count - 1;
                            }
                            _ => {}
                        }

                        let _ = http
                            .interaction_callback(
                                message_component.id,
                                message_component.token,
                                InteractionResponse::UpdateMessage(CallbackData {
                                    allowed_mentions: None,
                                    content: None,
                                    components: None,
                                    embeds: vec![pages[page_pointer].clone()],
                                    flags: None,
                                    tts: None,
                                }),
                            )
                            .await;
                    } else {
                        let _ = http
                            .interaction_callback(
                                message_component.id,
                                message_component.token.clone(),
                                InteractionResponse::DeferredUpdateMessage,
                            )
                            .await;
                        let _ = http.create_followup_message(message_component.token).unwrap()
                            .ephemeral(true)
                            .content("This view menu is only navigable by the original command invoker")
                            .await;
                    }
                }
            }
        }
        let _ = ctx.bot.http.delete_message(channel_id, message_id).await;
    }
    Ok(())
}

pub fn parse_username(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@!") {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if mention.starts_with("<@") {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    }
}

pub fn parse_role(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    }
}
