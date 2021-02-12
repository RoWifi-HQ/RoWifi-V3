use super::{activity, auto_detection};
use dashmap::DashSet;
use futures::{Future, FutureExt};
use rowifi_framework::{
    context::BotContext,
    prelude::{CommandError, EmbedExtensions, RoError},
};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::EmbedBuilder;
use twilight_gateway::Event;
use twilight_model::{
    channel::GuildChannel,
    guild::{GuildStatus, Permissions},
    id::{ChannelId, GuildId},
};

pub struct EventHandlerRef {
    unavailable: DashSet<GuildId>,
    auto_detection_started: AtomicBool,
    bot: BotContext,
}

#[derive(Clone)]
pub struct EventHandler(Arc<EventHandlerRef>);

impl EventHandler {
    pub fn new(bot: &BotContext) -> Self {
        Self {
            0: Arc::new(EventHandlerRef {
                bot: bot.to_owned(),
                unavailable: DashSet::new(),
                auto_detection_started: AtomicBool::new(false),
            }),
        }
    }
}

#[allow(clippy::type_complexity)]
impl Service<(u64, Event)> for EventHandler {
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, (shard_id, event): (u64, Event)) -> Self::Future {
        let eh = self.0.clone();

        async move {
            match &event {
                Event::GuildCreate(guild) => {
                    if eh.unavailable.contains(&guild.id) {
                        eh.unavailable.remove(&guild.id);
                        if eh.unavailable.is_empty()
                            && !eh.auto_detection_started.load(Ordering::SeqCst)
                            && shard_id % eh.bot.shards_per_cluster
                                == eh.bot.shards_per_cluster - 1
                        {
                            eh.auto_detection_started.store(true, Ordering::SeqCst);
                            let context_ad = eh.bot.clone();
                            tokio::spawn(async move {
                                auto_detection(context_ad).await;
                            });
                            let context_ac = eh.bot.clone();
                            tokio::spawn(async move {
                                activity(context_ac).await;
                            });
                        }
                    } else {
                        let content = "Thank you for adding RoWifi! To get started, please set up your server using `!setup`
                            \n\nTo get more information about announcements & updates, please join our support server\nhttps://www.discord.gg/h4BGGyR
                            \n\nTo view our documentation, please visit our website\nhttps://rowifi.link";
                        let mut channel = None;
                        for c in &guild.channels {
                            if let GuildChannel::Text(tc) = c {
                                if let Some(permissions) = eh.bot.cache.channel_permissions(tc.id) {
                                    if permissions.contains(Permissions::SEND_MESSAGES) {
                                        channel = Some(c);
                                        break;
                                    }
                                }
                            }
                        }
                        if let Some(channel) = channel {
                            let _ = eh.bot
                                .http
                                .create_message(channel.id())
                                .content(content)
                                .unwrap()
                                .await;
                        }
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title("Guild Join")
                            .unwrap()
                            .description(format!(
                                "Name: {}\nServer Id: {}\nOwner Id: {}\nMembercount: {}",
                                guild.name,
                                guild.id.0,
                                guild.owner_id.0,
                                guild.member_count.unwrap_or_default()
                            ))
                            .unwrap()
                            .build()
                            .unwrap();
                        eh.bot.log_debug(log_embed).await;
                    }
                }
                Event::GuildDelete(guild) => {
                    if guild.unavailable {
                        eh.unavailable.insert(guild.id);
                    } else {
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title("Guild Leave")
                            .unwrap()
                            .description(format!("Server Id: {}", guild.id.0))
                            .unwrap()
                            .build()
                            .unwrap();
                        eh.bot.log_debug(log_embed).await;
                    }
                }
                Event::Ready(ready) => {
                    tracing::info!("RoWifi ready for service!");
                    for status in &ready.guilds {
                        if let GuildStatus::Offline(ug) = status {
                            eh.unavailable.insert(ug.id);
                        }
                    }
                    let guild_ids = ready
                        .guilds
                        .iter()
                        .map(|k| match k {
                            GuildStatus::Offline(u) => u.id.0,
                            GuildStatus::Online(g) => g.id.0,
                        })
                        .collect::<Vec<u64>>();
                    let guilds = eh.bot.database.get_guilds(&guild_ids, false).await?;
                    for guild in guilds {
                        if let Some(command_prefix) = guild.command_prefix {
                            eh.bot.prefixes
                                .insert(GuildId(guild.id as u64), command_prefix);
                        }
                        for channel in guild.disabled_channels {
                            eh.bot.disabled_channels
                                .insert(ChannelId(channel as u64));
                        }
                    }
                }
                Event::UnavailableGuild(g) => {
                    eh.unavailable.insert(g.id);
                }
                Event::MemberAdd(m) => {
                    let server = match eh.bot.cache.guild(m.guild_id) {
                        Some(s) => s,
                        None => return Ok(()),
                    };
                    let member = match eh.bot.cache.member(m.guild_id, m.user.id) {
                        Some(m) => m,
                        None => return Ok(()),
                    };
                    let guild = match eh.bot.database.get_guild(m.guild_id.0).await? {
                        Some(g) => g,
                        None => return Ok(()),
                    };
                    if !guild.settings.update_on_join {
                        return Ok(());
                    }
                    let user = match eh.bot.database.get_user(m.user.id.0).await? {
                        Some(u) => u,
                        None => return Ok(()),
                    };
                    if server.owner_id == m.user.id {
                        return Ok(());
                    }
                    let guild_roles = eh.bot.cache.roles(m.guild_id);
                    let (added_roles, removed_roles, disc_nick) = match eh.bot
                        .update_user(member, &user, &server, &guild, &guild_roles)
                        .await
                    {
                        Ok(a) => a,
                        Err(e) => {
                            if let RoError::Command(CommandError::Blacklist(ref b)) = e {
                                if let Ok(channel) = eh.bot.http.create_private_channel(m.user.id).await {
                                    let _ = eh.bot
                                        .http
                                        .create_message(channel.id)
                                        .content(format!(
                                            "You were found on the server blacklist. Reason: {}",
                                            b
                                        ))
                                        .unwrap()
                                        .await;
                                }
                            }
                            return Err(e);
                        }
                    };
                    let log_embed = EmbedBuilder::new()
                        .default_data()
                        .title("Update On Join")
                        .unwrap()
                        .update_log(&added_roles, &removed_roles, &disc_nick)
                        .build()
                        .unwrap();
                    eh.bot.log_guild(m.guild_id, log_embed).await;
                }
                _ => {}
            }
            Ok(())
        }.boxed()
    }
}
