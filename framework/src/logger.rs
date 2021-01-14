use super::Context;
use twilight_model::{channel::embed::Embed, id::{WebhookId, GuildId}};
use twilight_util::link::webhook;

pub struct Logger {
    pub debug_webhook: (WebhookId, String),
    pub main_webhook: (WebhookId, String),
    pub premium_webhook: (WebhookId, String),
}

impl Logger {
    pub fn new(debug_webhook: &str, main_webhook: &str, premium_webhook: &str) -> Self {
        let (debug_id, debug_token) = webhook::parse(debug_webhook).unwrap();
        let (main_id, main_token) = webhook::parse(main_webhook).unwrap();
        let (premium_id, premium_token) = webhook::parse(premium_webhook).unwrap();

        Self {
            debug_webhook: (debug_id, debug_token.unwrap().to_owned()),
            main_webhook: (main_id, main_token.unwrap().to_owned()),
            premium_webhook: (premium_id, premium_token.unwrap().to_owned())
        }
    }

    pub async fn log_guild(&self, ctx: &Context, guild_id: GuildId, embed: Embed) {
        let log_channel = ctx.cache.guild(guild_id).and_then(|g| g.log_channel);
        if let Some(channel_id) = log_channel {
            let _ = ctx
                .http
                .create_message(channel_id)
                .embed(embed)
                .unwrap()
                .await;
        }
    }

    pub async fn log_debug(&self, ctx: &Context, text: &str) {
        let _ = ctx
            .http
            .execute_webhook(self.debug_webhook.0, self.debug_webhook.1.as_str())
            .content(text.to_string())
            .await;
    }

    pub async fn log_event(&self, ctx: &Context, embed: Embed) {
        let _ = ctx
            .http
            .execute_webhook(self.main_webhook.0, self.main_webhook.1.as_str())
            .embeds(vec![embed])
            .await;
    }

    pub async fn log_premium(&self, ctx: &Context, text: &str) {
        let _ = ctx
            .http
            .execute_webhook(self.premium_webhook.0, self.premium_webhook.1.as_str())
            .content(text.to_string())
            .await;
    }
}
