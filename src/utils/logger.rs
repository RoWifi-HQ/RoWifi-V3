use crate::framework::context::Context;
use twilight_model::id:: ChannelId;
use twilight_model::channel::embed::Embed;

#[derive(Clone, Default)]
pub struct Logger;

impl Logger {
    pub async fn log_guild(&self, ctx: &Context, log_channel: Option<ChannelId>, embed: Embed) {
        if let Some(channel_id) = log_channel {
            let _ = ctx.http.create_message(channel_id).embed(embed).unwrap().await;
        }
    }
}