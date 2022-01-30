use rowifi_models::discord::{
    application::component::Component,
    channel::{embed::Embed, Message},
};
use std::sync::atomic::Ordering;
use twilight_http::{
    request::{
        application::interaction::{CreateFollowupMessage, UpdateOriginalResponse},
        channel::message::create_message::CreateMessage,
        AttachmentFile,
    },
    response::ResponseFuture, client::InteractionClient,
};
use rowifi_models::discord::id::{Id, marker::ApplicationMarker};

use crate::{context::CommandContext, error::MessageError};

pub struct Responder<'a> {
    message: Option<CreateMessage<'a>>,
    interaction: Option<UpdateOriginalResponse<'a>>,
    followup: Option<CreateFollowupMessage<'a>>,
    interaction_client: InteractionClient<'a>
}

impl<'a> Responder<'a> {
    pub fn new(ctx: &'a CommandContext, application_id: Id<ApplicationMarker>) -> Self {
        let interaction_client = ctx.bot.http.interaction(application_id);
        ctx.interaction_token.as_ref().map_or_else(
            || Self {
                message: Some(ctx.bot.http.create_message(ctx.channel_id.0)),
                interaction: None,
                followup: None,
                interaction_client: ctx.bot.http.interaction(application_id)
            },
            move |interaction_token| { 
                if ctx.callback_invoked.load(Ordering::Relaxed) {   
                    Self {
                        message: None,
                        interaction: None,
                        followup: Some(
                            interaction_client
                                .create_followup_message(interaction_token),
                        ),
                        interaction_client
                    }
                } else {
                    ctx.callback_invoked.store(true, Ordering::Relaxed);
                    Self {
                        message: None,
                        interaction: Some(
                            interaction_client
                                .update_interaction_original(interaction_token),
                        ),
                        followup: None,
                        interaction_client
                    }
                }
            },
        )
    }

    pub fn content(mut self, content: &'a str) -> Result<Self, MessageError> {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.content(Some(content)).map_err(|e| MessageError::Interaction(e))?);
        } else if let Some(message) = self.message {
            self.message = Some(message.content(content).map_err(|e| MessageError::Create(e))?);
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.content(content).map_err(|e| MessageError::Followup(e))?);
        }
        Ok(self)
    }

    pub fn components(mut self, components: &'a [Component]) -> Result<Self, MessageError> {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.components(Some(components)).map_err(|e| MessageError::Interaction(e))?);
        } else if let Some(message) = self.message {
            self.message = Some(message.components(components).map_err(|e| MessageError::Create(e))?);
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.components(components).map_err(|e| MessageError::Followup(e))?);
        }
        Ok(self)
    }

    pub fn embeds(mut self, embeds: &'a [Embed]) -> Result<Self, MessageError> {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.embeds(Some(embeds)).map_err(|e| MessageError::Interaction(e))?);
        } else if let Some(message) = self.message {
            self.message = Some(message.embeds(embeds).map_err(|e| MessageError::Create(e))?);
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.embeds(embeds).map_err(|e| MessageError::Followup(e))?);
        }
        Ok(self)
    }

    #[must_use]
    pub fn files(mut self, files: &'a [AttachmentFile<'a>]) -> Self {
        if let Some(interaction) = self.interaction {
            self.interaction = Some(interaction.attach(files));
        } else if let Some(message) = self.message {
            self.message = Some(message.attach(files));
        } else if let Some(followup) = self.followup {
            self.followup = Some(followup.attach(files));
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
