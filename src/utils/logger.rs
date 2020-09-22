use crate::framework::context::Context;
use twilight_model::id:: GuildId;
use twilight_model::channel::embed::Embed;

#[derive(Clone, Default)]
pub struct Logger;

impl Logger {
    pub async fn log_guild(&self, ctx: &Context, guild_id: GuildId, embed: Embed) {
        let log_channel = ctx.cache.guild(guild_id).and_then(|g| g.log_channel);
        if let Some(channel_id) = log_channel {
            let _ = ctx.http.create_message(channel_id).embed(embed).unwrap().await;
        }
    }
}