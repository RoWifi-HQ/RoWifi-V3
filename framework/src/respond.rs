use rowifi_models::discord::{
    application::component::Component,
    channel::{embed::Embed, Message},
};
use std::sync::atomic::Ordering;
use twilight_http::{request::AttachmentFile, response::ResponseFuture};
use twilight_validate::message::{
    components as _components, content as _content, embeds as _embeds,
};

use crate::{context::CommandContext, error::MessageError};

pub struct Responder<'a> {
    ctx: &'a CommandContext,
    content: Option<&'a str>,
    components: Option<&'a [Component]>,
    embeds: Option<&'a [Embed]>,
    files: Option<&'a [AttachmentFile<'a>]>,
}

impl<'a> Responder<'a> {
    pub fn new(ctx: &'a CommandContext) -> Self {
        Self {
            ctx,
            content: None,
            components: None,
            embeds: None,
            files: None,
        }
    }

    pub fn content(mut self, content: &'a str) -> Result<Self, MessageError> {
        _content(content)?;

        self.content = Some(content);
        Ok(self)
    }

    pub fn components(mut self, components: &'a [Component]) -> Result<Self, MessageError> {
        _components(components)?;

        self.components = Some(components);
        Ok(self)
    }

    pub fn embeds(mut self, embeds: &'a [Embed]) -> Result<Self, MessageError> {
        _embeds(embeds)?;

        self.embeds = Some(embeds);
        Ok(self)
    }

    #[must_use]
    pub fn files(mut self, files: &'a [AttachmentFile<'a>]) -> Self {
        self.files = Some(files);
        self
    }

    pub fn exec(self) -> ResponseFuture<Message> {
        if let Some(interaction_token) = &self.ctx.interaction_token {
            if self.ctx.callback_invoked.load(Ordering::Relaxed) {
                let client = self.ctx.bot.http.interaction(self.ctx.bot.application_id);
                let mut req = client.create_followup_message(interaction_token);
                if let Some(content) = self.content {
                    req = req.content(content).unwrap();
                }
                if let Some(components) = self.components {
                    req = req.components(components).unwrap();
                }
                if let Some(embeds) = self.embeds {
                    req = req.embeds(embeds).unwrap();
                }
                if let Some(files) = self.files {
                    req = req.attach(files);
                }
                req.exec()
            } else {
                let client = self.ctx.bot.http.interaction(self.ctx.bot.application_id);
                let req = client
                    .update_interaction_original(interaction_token)
                    .content(self.content)
                    .unwrap()
                    .components(self.components)
                    .unwrap()
                    .embeds(self.embeds)
                    .unwrap()
                    .attach(self.files.unwrap_or_default());
                req.exec()
            }
        } else {
            let mut req = self.ctx.bot.http.create_message(self.ctx.channel_id.0);
            if let Some(content) = self.content {
                req = req.content(content).unwrap();
            }
            if let Some(components) = self.components {
                req = req.components(components).unwrap();
            }
            if let Some(embeds) = self.embeds {
                req = req.embeds(embeds).unwrap();
            }
            if let Some(files) = self.files {
                req = req.attach(files);
            }
            req.exec()
        }
    }
}
