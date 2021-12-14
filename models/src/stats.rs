use prometheus::{IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry};
use std::collections::HashMap;
use twilight_model::gateway::event::Event;

pub struct EventStats {
    pub ban_add: IntCounter,
    pub ban_remove: IntCounter,
    pub channel_create: IntCounter,
    pub channel_delete: IntCounter,
    pub channel_update: IntCounter,
    pub gateway_reconnect: IntCounter,
    pub guild_create: IntCounter,
    pub guild_delete: IntCounter,
    pub guild_update: IntCounter,
    pub member_add: IntCounter,
    pub member_remove: IntCounter,
    pub member_update: IntCounter,
    pub member_chunk: IntCounter,
    pub message_create: IntCounter,
    pub message_delete: IntCounter,
    pub message_delete_bulk: IntCounter,
    pub message_update: IntCounter,
    pub reaction_add: IntCounter,
    pub reaction_remove: IntCounter,
    pub reaction_remove_all: IntCounter,
    pub role_create: IntCounter,
    pub role_delete: IntCounter,
    pub role_update: IntCounter,
    pub unavailable_guild: IntCounter,
    pub user_update: IntCounter,
}

pub struct ResourceCounters {
    pub guilds: IntGauge,
    pub users: IntGauge,
}

pub struct BotStats {
    pub registry: Registry,
    pub event_counts: EventStats,
    pub resource_counts: ResourceCounters,
    pub command_counts: IntCounterVec,
    pub update_user: IntCounter,
}

impl BotStats {
    #[must_use]
    pub fn new(cluster_id: u64) -> Self {
        let event_counter = IntCounterVec::new(
            Opts::new("discord_events", "Events given by discord"),
            &["events"],
        )
        .unwrap();
        let resource_counter = IntGaugeVec::new(
            Opts::new("resource_counts", "Counts of all resource"),
            &["count"],
        )
        .unwrap();
        let command_counts =
            IntCounterVec::new(Opts::new("commands", "Executed commands"), &["name"]).unwrap();
        let update_user =
            IntCounter::with_opts(Opts::new("update_user", "Counts of any user updated")).unwrap();

        let mut static_labels = HashMap::new();
        static_labels.insert(String::from("cluster"), cluster_id.to_string());
        let registry = Registry::new_custom(Some("rowifi".into()), Some(static_labels)).unwrap();

        registry.register(Box::new(event_counter.clone())).unwrap();
        registry
            .register(Box::new(resource_counter.clone()))
            .unwrap();
        registry.register(Box::new(command_counts.clone())).unwrap();
        registry.register(Box::new(update_user.clone())).unwrap();

        BotStats {
            registry,
            event_counts: EventStats {
                ban_add: event_counter
                    .get_metric_with_label_values(&["BanAdd"])
                    .unwrap(),
                ban_remove: event_counter
                    .get_metric_with_label_values(&["BanRemove"])
                    .unwrap(),
                channel_create: event_counter
                    .get_metric_with_label_values(&["ChannelCreate"])
                    .unwrap(),
                channel_delete: event_counter
                    .get_metric_with_label_values(&["ChannelDelete"])
                    .unwrap(),
                channel_update: event_counter
                    .get_metric_with_label_values(&["ChannelUpdate"])
                    .unwrap(),
                gateway_reconnect: event_counter
                    .get_metric_with_label_values(&["GatewayReconnect"])
                    .unwrap(),
                guild_create: event_counter
                    .get_metric_with_label_values(&["GuildCreate"])
                    .unwrap(),
                guild_delete: event_counter
                    .get_metric_with_label_values(&["GuildDelete"])
                    .unwrap(),
                guild_update: event_counter
                    .get_metric_with_label_values(&["GuildUpdate"])
                    .unwrap(),
                member_add: event_counter
                    .get_metric_with_label_values(&["MemberAdd"])
                    .unwrap(),
                member_remove: event_counter
                    .get_metric_with_label_values(&["MemberRemove"])
                    .unwrap(),
                member_chunk: event_counter
                    .get_metric_with_label_values(&["MemberChunk"])
                    .unwrap(),
                member_update: event_counter
                    .get_metric_with_label_values(&["MemberUpdate"])
                    .unwrap(),
                message_create: event_counter
                    .get_metric_with_label_values(&["MessageCreate"])
                    .unwrap(),
                message_delete: event_counter
                    .get_metric_with_label_values(&["MessageDelete"])
                    .unwrap(),
                message_delete_bulk: event_counter
                    .get_metric_with_label_values(&["MessageDeleteBulk"])
                    .unwrap(),
                message_update: event_counter
                    .get_metric_with_label_values(&["MessageUpdate"])
                    .unwrap(),
                reaction_add: event_counter
                    .get_metric_with_label_values(&["ReactionAdd"])
                    .unwrap(),
                reaction_remove: event_counter
                    .get_metric_with_label_values(&["ReactionRemove"])
                    .unwrap(),
                reaction_remove_all: event_counter
                    .get_metric_with_label_values(&["ReactionRemoveAll"])
                    .unwrap(),
                role_create: event_counter
                    .get_metric_with_label_values(&["RoleCreate"])
                    .unwrap(),
                role_delete: event_counter
                    .get_metric_with_label_values(&["RoleDelete"])
                    .unwrap(),
                role_update: event_counter
                    .get_metric_with_label_values(&["RoleUpdate"])
                    .unwrap(),
                unavailable_guild: event_counter
                    .get_metric_with_label_values(&["UnavailableGuild"])
                    .unwrap(),
                user_update: event_counter
                    .get_metric_with_label_values(&["UserUpdate"])
                    .unwrap(),
            },
            resource_counts: ResourceCounters {
                guilds: resource_counter
                    .get_metric_with_label_values(&["Guilds"])
                    .unwrap(),
                users: resource_counter
                    .get_metric_with_label_values(&["Users"])
                    .unwrap(),
            },
            command_counts,
            update_user,
        }
    }

    pub fn update(&self, event: &Event) {
        match event {
            Event::BanAdd(_) => self.event_counts.ban_add.inc(),
            Event::BanRemove(_) => self.event_counts.ban_remove.inc(),
            Event::ChannelCreate(_) => self.event_counts.channel_create.inc(),
            Event::ChannelDelete(_) => self.event_counts.channel_delete.inc(),
            Event::ChannelUpdate(_) => self.event_counts.channel_update.inc(),
            Event::GatewayReconnect => self.event_counts.gateway_reconnect.inc(),
            Event::GuildCreate(_) => self.event_counts.guild_create.inc(),
            Event::GuildDelete(_) => self.event_counts.guild_delete.inc(),
            Event::GuildUpdate(_) => self.event_counts.guild_update.inc(),
            Event::MemberAdd(_) => self.event_counts.member_add.inc(),
            Event::MemberRemove(_) => self.event_counts.member_remove.inc(),
            Event::MemberUpdate(_) => self.event_counts.member_update.inc(),
            Event::MemberChunk(_) => self.event_counts.member_chunk.inc(),
            Event::MessageCreate(_) => self.event_counts.message_create.inc(),
            Event::MessageDelete(_) => self.event_counts.message_delete.inc(),
            Event::MessageDeleteBulk(_) => self.event_counts.message_delete_bulk.inc(),
            Event::MessageUpdate(_) => self.event_counts.message_update.inc(),
            Event::ReactionAdd(_) => self.event_counts.reaction_add.inc(),
            Event::ReactionRemove(_) => self.event_counts.reaction_remove.inc(),
            Event::ReactionRemoveAll(_) => self.event_counts.reaction_remove_all.inc(),
            Event::RoleCreate(_) => self.event_counts.role_create.inc(),
            Event::RoleDelete(_) => self.event_counts.role_delete.inc(),
            Event::RoleUpdate(_) => self.event_counts.role_update.inc(),
            Event::UnavailableGuild(_) => self.event_counts.unavailable_guild.inc(),
            Event::UserUpdate(_) => self.event_counts.user_update.inc(),
            _ => {}
        }
    }
}
