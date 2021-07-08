use futures_util::future::FutureExt;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_http::{
    request::{application::UpdateOriginalResponse, prelude::CreateMessage},
    Error as DiscordHttpError,
};
use twilight_model::{application::component::Component, channel::embed::Embed, id::MessageId};

use crate::context::CommandContext;

pub struct Responder<'a> {
    message: Option<CreateMessage<'a>>,
    interaction: Option<UpdateOriginalResponse<'a>>,
}

impl<'a> Responder<'a> {
    pub fn new(ctx: &'a CommandContext) -> Self {
        ctx.interaction_token.as_ref().map_or_else(
            || Self {
                message: Some(ctx.bot.http.create_message(ctx.channel_id)),
                interaction: None,
            },
            |interaction_token| Self {
                message: None,
                interaction: Some(
                    ctx.bot
                        .http
                        .update_interaction_original(interaction_token)
                        .unwrap(),
                ),
            },
        )
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.content(Some(content)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.content(content).unwrap());
        }
        self
    }

    pub fn component(mut self, component: Component) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.component(component).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.component(component).unwrap());
        }
        self
    }

    pub fn components(mut self, components: Vec<Component>) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.components(components).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.components(components).unwrap());
        }
        self
    }

    pub fn embed(mut self, embed: Embed) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.embeds(Some(vec![embed])).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.embed(embed).unwrap());
        }
        self
    }

    pub fn file(mut self, name: impl Into<String>, file: impl Into<Vec<u8>>) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.file(name, file));
        } else if let Some(message) = self.message {
            self.message = Some(message.file(name, file));
        }
        self
    }
}

#[allow(clippy::option_if_let_else)]
impl Future for Responder<'_> {
    type Output = Result<Option<MessageId>, DiscordHttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(interaction) = self.interaction.as_mut() {
            interaction.poll_unpin(cx).map(|i| i.map(|_| None))
        } else if let Some(message) = self.message.as_mut() {
            message.poll_unpin(cx).map(|p| p.map(|m| Some(m.id)))
        } else {
            Poll::Ready(Ok(None))
        }
    }
}
