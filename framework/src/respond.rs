use futures_util::future::FutureExt;
use rowifi_models::discord::{
    application::component::Component, channel::embed::Embed, id::MessageId,
};
use std::{
    future::Future,
    pin::Pin,
    sync::atomic::Ordering,
    task::{Context, Poll},
};
use twilight_http::{
    request::{
        application::{CreateFollowupMessage, UpdateOriginalResponse},
        prelude::CreateMessage,
    },
    Error as DiscordHttpError,
};

use crate::context::CommandContext;

pub struct Responder<'a> {
    message: Option<CreateMessage<'a>>,
    interaction: Option<UpdateOriginalResponse<'a>>,
    followup: Option<CreateFollowupMessage<'a>>,
}

impl<'a> Responder<'a> {
    pub fn new(ctx: &'a CommandContext) -> Self {
        ctx.interaction_token.as_ref().map_or_else(
            || Self {
                message: Some(ctx.bot.http.create_message(ctx.channel_id)),
                interaction: None,
                followup: None,
            },
            |interaction_token| {
                if ctx.callback_invoked.load(Ordering::Relaxed) {
                    Self {
                        message: None,
                        interaction: None,
                        followup: Some(
                            ctx.bot
                                .http
                                .create_followup_message(interaction_token)
                                .unwrap(),
                        ),
                    }
                } else {
                    ctx.callback_invoked.store(true, Ordering::Relaxed);
                    Self {
                        message: None,
                        interaction: Some(
                            ctx.bot
                                .http
                                .update_interaction_original(interaction_token)
                                .unwrap(),
                        ),
                        followup: None,
                    }
                }
            },
        )
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        let content = content.into();
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.content(Some(content)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.content(content).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.content(content));
        }
        self
    }

    pub fn component(mut self, component: Component) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.components(Some(vec![component])).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.components(vec![component]).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.components(vec![component]).unwrap());
        }
        self
    }

    pub fn components(mut self, components: Vec<Component>) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.components(Some(components)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.components(components).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.components(components).unwrap());
        }
        self
    }

    pub fn embed(mut self, embed: Embed) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.embeds(Some(vec![embed])).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.embeds(vec![embed]).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.embeds(vec![embed]));
        }
        self
    }
    pub fn embeds(mut self, embeds: Vec<Embed>) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.embeds(Some(embeds)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.embeds(embeds).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.embeds(embeds));
        }
        self
    }

    pub fn file(mut self, name: impl Into<String>, file: impl Into<Vec<u8>>) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.files(vec![(name, file)]));
        } else if let Some(message) = self.message {
            self.message = Some(message.files(vec![(name, file)]));
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.files(vec![(name, file)]));
        }
        self
    }
}

#[allow(clippy::option_if_let_else)]
impl Future for Responder<'_> {
    type Output = Result<Option<MessageId>, DiscordHttpError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(interaction) = self.interaction.as_mut() {
            interaction.poll_unpin(cx).map(|i| i.map(|m| Some(m.id)))
        } else if let Some(message) = self.message.as_mut() {
            message.poll_unpin(cx).map(|p| p.map(|m| Some(m.id)))
        } else if let Some(followup) = self.followup.as_mut() {
            followup.poll_unpin(cx).map(|p| p.map(|m| m.map(|i| i.id)))
        } else {
            Poll::Ready(Ok(None))
        }
    }
}
