use rowifi_models::discord::{application::component::Component, channel::{Message, embed::Embed}};
use std::sync::atomic::Ordering;
use twilight_http::{request::{
        application::{CreateFollowupMessage, UpdateOriginalResponse},
        prelude::CreateMessage,
    }, response::ResponseFuture};

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

    pub fn content(mut self, content: &'a str) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.content(Some(&content)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.content(&content).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.content(&content));
        }
        self
    }

    pub fn components(mut self, components: &'a [Component]) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.components(Some(components)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.components(components).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.components(components).unwrap());
        }
        self
    }

    pub fn embeds(mut self, embeds: &'a [Embed]) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.embeds(Some(embeds)).unwrap());
        } else if let Some(message) = self.message {
            self.message = Some(message.embeds(embeds).unwrap());
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.embeds(embeds));
        }
        self
    }

    pub fn files(mut self, files: &'a [(&'a str, &'a [u8])]) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.files(files));
        } else if let Some(message) = self.message {
            self.message = Some(message.files(files));
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.files(files));
        }
        self
    }

    pub fn exec(self) -> ResponseFuture<Message> {
        if let Some(interaction) = self.interaction {
            interaction.exec()
        } else if let Some(message) = self.message {
            message.exec()
        } else if let Some(followup) = self.followup {
            followup.exec()
        } else {
            unreachable!()
        }
    }
}