use dashmap::DashSet;
use futures_util::future::{Future, FutureExt};
use itertools::Itertools;
use rowifi_framework::{context::BotContext, prelude::*};
use rowifi_models::{
    bind::Bind,
    discord::{channel::GuildChannel, guild::Permissions},
    guild::{GuildType, RoGuild},
    id::{ChannelId, GuildId, UserId},
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
use twilight_gateway::Event;

use crate::{
    services::auto_detection,
    utils::{UpdateUser, UpdateUserResult},
};

use super::activity;

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
                bot: bot.clone(),
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
                    if eh.unavailable.contains(&GuildId(guild.id)) {
                        eh.unavailable.remove(&GuildId(guild.id));
                    } else {
                        let content = "Thank you for adding RoWifi! To view our setup guide, check out our post: https://rowifi.link/blog/setup
                            \nTo get more information about announcements & updates, please join our support server: https://www.discord.gg/h4BGGyR
                            \nTo view our documentation, please visit our website: https://rowifi.link";
                        let mut channel = None;
                        for c in &guild.channels {
                            if let GuildChannel::Text(tc) = c {
                                if let Some(permissions) = eh.bot.cache.channel_permissions(ChannelId(tc.id)) {
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
                                .exec()
                                .await;
                        }
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title("Guild Join")
                            .description(format!(
                                "Name: {}\nServer Id: {}\nOwner Id: {}\nMembercount: {}",
                                guild.name,
                                guild.id.0,
                                guild.owner_id.0,
                                guild.member_count.unwrap_or_default()
                            ))
                            .build()
                            .unwrap();
                        eh.bot.log_debug(log_embed).await;
                    }
                }
                Event::GuildDelete(guild) => {
                    if guild.unavailable {
                        eh.unavailable.insert(GuildId(guild.id));
                    } else {
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title("Guild Leave")
                            .description(format!("Server Id: {}", guild.id.0))
                            .build()
                            .unwrap();
                        eh.bot.log_debug(log_embed).await;
                    }
                }
                Event::Ready(ready) => {
                    tracing::info!("RoWifi ready for service!");
                    for ug in &ready.guilds {
                        eh.unavailable.insert(GuildId(ug.id));
                    }
                    if !eh.auto_detection_started.load(Ordering::SeqCst)
                        && shard_id % eh.bot.shards_per_cluster
                            == eh.bot.shards_per_cluster - 1
                    {
                        eh.auto_detection_started.store(true, Ordering::SeqCst);
                        let context_ad = eh.bot.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(30)).await;
                            auto_detection::auto_detection(context_ad).await;
                        });
                        let context_ac = eh.bot.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(3 * 60)).await;
                            activity(context_ac).await;
                        });
                    }
                    let guild_ids = ready
                        .guilds
                        .iter()
                        .map(|k| k.id.0.get() as i64)
                        .collect::<Vec<_>>();
                    let guilds = eh.bot.database.query::<RoGuild>("SELECT * FROM guilds WHERE guild_id = ANY($1)", &[&guild_ids]).await?;
                    for guild in guilds {
                        let guild_id = guild.guild_id;
                        eh.bot.prefixes.insert(guild_id, guild.command_prefix);
                        for channel in guild.disabled_channels {
                            eh.bot.disabled_channels
                                .insert(channel);
                        }

                        if guild.kind != GuildType::Free {
                            eh.bot.admin_roles.insert(guild_id, guild.admin_roles);
                            eh.bot.trainer_roles.insert(guild_id, guild.trainer_roles);
                            eh.bot.bypass_roles.insert(guild_id, guild.bypass_roles);
                            eh.bot.nickname_bypass_roles.insert(guild_id, guild.nickname_bypass_roles);
                        }

                        if let Some(log_channel) = guild.log_channel {
                            eh.bot.log_channels.insert(guild_id, log_channel);
                        }
                    }
                }
                Event::UnavailableGuild(g) => {
                    eh.unavailable.insert(GuildId(g.id));
                }
                Event::MemberAdd(m) => {
                    let guild_id = GuildId(m.guild_id);
                    let user_id = UserId(m.user.id);
                    let server = match eh.bot.cache.guild(guild_id) {
                        Some(s) => s,
                        None => return Ok(()),
                    };
                    let member = match eh.bot.cache.member(guild_id, user_id) {
                        Some(m) => m,
                        None => return Ok(()),
                    };
                    let guild = eh.bot.database.get_guild(guild_id).await?;
                    if !guild.update_on_join {
                        return Ok(());
                    }
                    let user = match eh.bot.database.get_linked_user(user_id, guild_id).await? {
                        Some(u) => u,
                        None => {
                            if let Some(verification_role) = guild.verification_roles.get(0) {
                                if let Some(role) = eh.bot.cache.role(*verification_role) {
                                    eh.bot.http.add_guild_member_role(m.guild_id, user_id.0, role.id.0).exec().await?;
                                }
                            }
                            return Ok(());
                        },
                    };

                    let guild_roles = eh.bot.cache.roles(guild_id);

                    let binds = eh.bot
                        .database
                        .query::<Bind>(
                            "SELECT * FROM binds WHERE guild_id = $1",
                            &[&guild.guild_id],
                        )
                        .await?;
                    let all_roles = binds
                        .iter()
                        .flat_map(|b| b.discord_roles())
                        .unique()
                        .collect::<Vec<_>>();

                    let update_user = UpdateUser {
                        ctx: &eh.bot,
                        member: &member,
                        user: &user,
                        server: &server,
                        guild: &guild,
                        binds: &binds,
                        guild_roles: &guild_roles,
                        bypass_roblox_cache: false,
                        all_roles: &all_roles,
                    };
                    let (added_roles, removed_roles, disc_nick) = match update_user.execute().await
                    {
                        UpdateUserResult::Success(a, r, n) => (a, r, n),
                        UpdateUserResult::Blacklist(reason) => {
                            if let Ok(channel) = eh.bot.http.create_private_channel(m.user.id).exec().await?.model().await {
                                let _ = eh.bot
                                    .http
                                    .create_message(channel.id)
                                    .content(&format!(
                                        "You were found on the server blacklist. Reason: {}",
                                        reason
                                    ))
                                    .unwrap()
                                    .exec()
                                    .await;
                            }
                            return Ok(());
                        },
                        UpdateUserResult::InvalidNickname(_) => return Ok(()),
                        UpdateUserResult::Error(err) => return Err(err)
                    };
                    let log_embed = EmbedBuilder::new()
                        .default_data()
                        .title("Update On Join")
                        .update_log(&added_roles, &removed_roles, &disc_nick)
                        .build()
                        .unwrap();
                    eh.bot.log_guild(guild_id, log_embed).await;
                }
                _ => {}
            }
            Ok(())
        }.boxed()
    }
}
