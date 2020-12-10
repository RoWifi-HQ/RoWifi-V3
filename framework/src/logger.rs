use super::Context;
use twilight_model::channel::embed::Embed;
use twilight_model::id::GuildId;

pub struct Logger {
    pub debug_webhook: String,
    pub main_webhook: String,
    pub premium_webhook: String,
}

impl Logger {
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
            .execute_webhook_from_url(&self.debug_webhook)
            .unwrap()
            .content(text.to_string())
            .await;
    }

    pub async fn log_event(&self, ctx: &Context, embed: Embed) {
        let _ = ctx
            .http
            .execute_webhook_from_url(&self.main_webhook)
            .unwrap()
            .embeds(vec![embed])
            .await;
    }

    pub async fn log_premium(&self, ctx: &Context, text: &str) {
        let _ = ctx
            .http
            .execute_webhook_from_url(&self.premium_webhook)
            .unwrap()
            .content(text.to_string())
            .await;
    }
}
