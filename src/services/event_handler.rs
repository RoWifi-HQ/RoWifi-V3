use crate::framework::prelude::Context;
use std::sync::Arc;
use twilight_gateway::Event;
use twilight_model::{
    id::GuildId,
    gateway::payload::RequestGuildMembers,
    guild::GuildStatus
};
use dashmap::DashSet;

#[derive(Default)]
pub struct EventHandlerRef {
    unavailable: DashSet<GuildId>
}

#[derive(Default, Clone)]
pub struct EventHandler(Arc<EventHandlerRef>);

impl EventHandler {
    pub async fn handle_event(&self, shard_id: u64, event: &Event, ctx: Context) {
        match &event {
            Event::GuildCreate(guild) => {
                self.0.unavailable.remove(&guild.id);
                let req = RequestGuildMembers::builder(guild.id).query("", None);
                let _res = ctx.cluster.command(shard_id, &req).await;
            },
            Event::Ready(ready) => {
                tracing::debug!("RoWifi ready for service!");
                for status in ready.guilds.values() {
                    if let GuildStatus::Offline(ug) = status {
                        self.0.unavailable.insert(ug.id);
                    }
                }
            },
            Event::UnavailableGuild(g) => {
                self.0.unavailable.insert(g.id);
            }
            _ => {}
        }
    }
}