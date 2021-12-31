use crate::{
    context::CommandContext,
    error::{CommandError, RoError},
    extensions::StandbyExtensions,
};

use rowifi_models::{
    bind::Template,
    discord::{
        application::{
            callback::{CallbackData, InteractionResponse},
            component::{
                action_row::ActionRow,
                button::{Button, ButtonStyle},
                Component, SelectMenu,
            },
            interaction::Interaction,
        },
        channel::{embed::Embed, ReactionType},
        gateway::event::Event,
    }, id::{RoleId, UserId, ChannelId},
};
use std::{
    cmp::{max, min},
    num::ParseIntError,
    str::FromStr,
    time::Duration,
};
use tokio_stream::StreamExt;

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

pub async fn await_confirmation(question: &str, ctx: &CommandContext) -> Result<bool, RoError> {
    let message = ctx
        .respond()
        .content(question)?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some("confirm-yes".into()),
                    disabled: false,
                    emoji: None,
                    label: Some("Yes".into()),
                    style: ButtonStyle::Primary,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: Some("confirm-no".into()),
                    disabled: false,
                    emoji: None,
                    label: Some("No".into()),
                    style: ButtonStyle::Danger,
                    url: None,
                }),
            ],
        })])?
        .exec()
        .await?
        .model()
        .await?;

    let message_id = message.id;
    let author_id = ctx.author.id;

    let mut answer = false;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);

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
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await?;
                    if message_component.data.custom_id == "confirm-yes" {
                        answer = true;
                        break;
                    } else if message_component.data.custom_id == "confirm-no" {
                        answer = false;
                        break;
                    }
                }
                let _ = ctx
                    .bot
                    .http
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
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .exec()
                    .await;
            }
        }
    }

    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(answer)
}

pub async fn await_template_reply(
    question: &str,
    ctx: &CommandContext,
    mut select_menu: SelectMenu,
) -> Result<Template, RoError> {
    let select_custom_id = select_menu.custom_id.clone();
    let message = ctx
        .respond()
        .content(question)?
        .components(&[
            Component::ActionRow(ActionRow {
                components: vec![Component::SelectMenu(select_menu.clone())],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some("template-reply-cancel".into()),
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

    let message_id = message.id;
    let author_id = ctx.author.id;

    select_menu.disabled = true;

    let stream = ctx
        .bot
        .standby
        .wait_for_event_stream(move |event: &Event| {
            if let Event::InteractionCreate(interaction) = &event {
                if let Interaction::MessageComponent(message_component) = &interaction.0 {
                    if message_component.message.id == message_id {
                        return true;
                    }
                }
            } else if let Event::MessageCreate(msg) = &event {
                if msg.author.id == author_id && !msg.content.is_empty() {
                    return true;
                }
            }
            false
        })
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);
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
                            &InteractionResponse::UpdateMessage(CallbackData {
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
                        .exec()
                        .await?;
                    ctx.bot.ignore_message_components.remove(&message_id);
                    if message_component.data.custom_id == "template-reply-cancel" {
                        ctx.bot
                            .http
                            .create_followup_message(&message_component.token)
                            .unwrap()
                            .content("Command has been cancelled")
                            .exec()
                            .await?;
                        return Err(CommandError::Cancelled.into());
                    } else if message_component.data.custom_id == select_custom_id {
                        let template_str = message_component.data.values[0].clone();
                        return Ok(Template(template_str));
                    }
                }
                let _ = ctx
                    .bot
                    .http
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
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .exec()
                    .await;
            }
        } else if let Event::MessageCreate(msg) = &event {
            ctx.bot.ignore_message_components.remove(&message_id);
            return Ok(Template(msg.content.clone()));
        }
    }

    ctx.bot.ignore_message_components.remove(&message_id);
    Err(CommandError::Timeout.into())
}

pub async fn await_reply(question: &str, ctx: &CommandContext) -> Result<String, RoError> {
    let message = ctx
        .respond()
        .content(question)?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some("reply-cancel".into()),
                disabled: false,
                emoji: None,
                label: Some("Cancel".into()),
                style: ButtonStyle::Danger,
                url: None,
            })],
        })])?
        .exec()
        .await?
        .model()
        .await?;
    let message_id = message.id;
    let author_id = ctx.author.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_event_stream(move |event: &Event| {
            if let Event::InteractionCreate(interaction) = &event {
                if let Interaction::MessageComponent(message_component) = &interaction.0 {
                    if message_component.message.id == message_id {
                        return true;
                    }
                }
            } else if let Event::MessageCreate(msg) = &event {
                if msg.author.id == author_id && !msg.content.is_empty() {
                    return true;
                }
            }
            false
        })
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id
                    && message_component.data.custom_id == "reply-cancel"
                {
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await?;
                    ctx.bot
                        .http
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .content("Command has been cancelled")
                        .exec()
                        .await?;
                    ctx.bot.ignore_message_components.remove(&message_id);
                    return Err(CommandError::Cancelled.into());
                }
                let _ = ctx
                    .bot
                    .http
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
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .exec()
                    .await;
            }
        } else if let Event::MessageCreate(msg) = &event {
            ctx.bot.ignore_message_components.remove(&message_id);
            return Ok(msg.content.clone());
        }
    }

    ctx.bot.ignore_message_components.remove(&message_id);
    Err(CommandError::Timeout.into())
}

pub async fn paginate_embed(
    ctx: &CommandContext,
    pages: Vec<Embed>,
    page_count: usize,
) -> Result<(), RoError> {
    let page_count = page_count;
    if page_count <= 1 {
        ctx.respond().embeds(&[pages[0].clone()])?.exec().await?;
    } else {
        let message = ctx
            .respond()
            .embeds(&[pages[0].clone()])?
            .components(&[Component::ActionRow(ActionRow {
                components: vec![
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ReactionType::Unicode {
                            name: "⏮️".into()
                        }),
                        label: Some("First Page".into()),
                        custom_id: Some("first-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ReactionType::Unicode {
                            name: "◀️".into()
                        }),
                        label: Some("Previous Page".into()),
                        custom_id: Some("previous-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ReactionType::Unicode {
                            name: "▶️".into()
                        }),
                        label: Some("Next Page".into()),
                        custom_id: Some("next-page".into()),
                        url: None,
                        disabled: false,
                    }),
                    Component::Button(Button {
                        style: ButtonStyle::Primary,
                        emoji: Some(ReactionType::Unicode {
                            name: "⏭️".into()
                        }),
                        label: Some("Last Page".into()),
                        custom_id: Some("last-page".into()),
                        url: None,
                        disabled: false,
                    }),
                ],
            })])?
            .exec()
            .await?
            .model()
            .await?;

        //Get some easy named vars
        let message_id = message.id;
        let author_id = ctx.author.id;
        let http = ctx.bot.http.clone();

        let component_interaction = ctx
            .bot
            .standby
            .wait_for_component_interaction(message_id)
            .timeout(Duration::from_secs(300));
        tokio::pin!(component_interaction);

        ctx.bot.ignore_message_components.insert(message_id);
        let mut page_pointer: usize = 0;
        while let Some(Ok(event)) = component_interaction.next().await {
            if let Event::InteractionCreate(interaction) = event {
                if let Interaction::MessageComponent(message_component) = interaction.0 {
                    let component_interaction_author = message_component.author_id().unwrap();
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
                                &message_component.token,
                                &InteractionResponse::UpdateMessage(CallbackData {
                                    allowed_mentions: None,
                                    content: None,
                                    components: None,
                                    embeds: vec![pages[page_pointer].clone()],
                                    flags: None,
                                    tts: None,
                                }),
                            )
                            .exec()
                            .await;
                    } else {
                        let _ = http
                            .interaction_callback(
                                message_component.id,
                                &message_component.token,
                                &InteractionResponse::DeferredUpdateMessage,
                            )
                            .exec()
                            .await;
                        let _ = http
                            .create_followup_message(&message_component.token)
                            .unwrap()
                            .ephemeral(true)
                            .content(
                                "This view menu is only navigable by the original command invoker",
                            )
                            .exec()
                            .await;
                    }
                }
            }
        }
        ctx.bot.ignore_message_components.remove(&message_id);
    }
    Ok(())
}

pub fn parse_username(mention: impl AsRef<str>) -> Option<UserId> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@!") {
        let len = mention.len() - 1;
        let id = mention[3..len].parse::<u64>().ok();
        id.map(|i| UserId::new(i))
    } else if mention.starts_with("<@") {
        let len = mention.len() - 1;
        let id = mention[2..len].parse::<u64>().ok();
        id.map(|i| UserId::new(i))
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(UserId::new(r))
    } else {
        None
    }
}

pub fn parse_role(mention: impl AsRef<str>) -> Option<RoleId> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        let id = mention[3..len].parse::<u64>().ok();
        id.map(|i| RoleId::new(i))
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(RoleId::new(r))
    } else {
        None
    }
}

pub fn parse_channel(mention: impl AsRef<str>) -> Option<ChannelId> {
    let mention = mention.as_ref();

    if mention.len() < 3 {
        return None;
    }

    if mention.starts_with("<#") && mention.ends_with('>') {
        let len = mention.len() - 1;
        let id = mention[2..len].parse::<u64>().ok();
        id.map(|i| ChannelId::new(i))
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(ChannelId::new(r))
    } else {
        None
    }
}

pub enum RankId {
    Range(i64, i64),
    Single(i64),
}

impl FromStr for RankId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits = s.split('-').collect::<Vec<_>>();
        if splits.len() == 2 {
            if let Ok(r1) = splits[0].parse::<i64>() {
                if let Ok(r2) = splits[1].parse::<i64>() {
                    return Ok(Self::Range(r1, r2));
                }
            }
        }
        let r = s.parse::<i64>()?;
        Ok(Self::Single(r))
    }
}
