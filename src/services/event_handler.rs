use crate::framework::prelude::Context;
use crate::utils::{misc::{channel_permissions, EmbedExtensions}, error::RoError};
use std::sync::Arc;
use twilight_gateway::Event;
use twilight_model::{
    id::{GuildId, ChannelId},
    gateway::payload::RequestGuildMembers,
    guild::{GuildStatus, Permissions},
    channel::GuildChannel
};
use twilight_embed_builder::EmbedBuilder;
use dashmap::DashSet;

#[derive(Default)]
pub struct EventHandlerRef {
    unavailable: DashSet<GuildId>
}

#[derive(Default, Clone)]
pub struct EventHandler(Arc<EventHandlerRef>);

impl EventHandler {
    pub async fn handle_event(&self, shard_id: u64, event: &Event, ctx: &Context) -> Result<(), RoError> {
        match &event {
            Event::GuildCreate(guild) => {
                if self.0.unavailable.contains(&guild.id) {
                    self.0.unavailable.remove(&guild.id);
                    let req = RequestGuildMembers::builder(guild.id).query("", None);
                    let _res = ctx.cluster.command(shard_id, &req).await;
                } else {
                    let current_user = ctx.cache.current_user().unwrap();
                    let current_member = ctx.cache.member(guild.id, current_user.id).unwrap();
                    let content = "Thank you for adding RoWifi! To get started, please set up your server using `!setup`
                        \n\nTo get more information about announcements & updates, please join our support server\nhttps://www.discord.gg/h4BGGyR
                        \n\nTo view our documentation, please visit our website\nhttps://rowifi.link";
                    let mut channel = None;
                    for c in guild.channels.values() {
                        if let GuildChannel::Text(tc) = c {
                            match channel_permissions(&ctx, guild.id, current_member.user.id, &current_member.roles, tc.id) {
                                Ok(permissions) =>  {
                                    if permissions.contains(Permissions::SEND_MESSAGES) {
                                        channel = Some(c);
                                        break;
                                    }
                                },
                                Err(why) => tracing::error!(guild_id = ?guild.id, channel_id = ?tc.id, reason = ?why)
                            } 
                        }
                    }
                    if let Some(channel) = channel {
                        let _ = ctx.http.create_message(channel.id()).content(content).unwrap().await;
                    }
                    let log_embed = EmbedBuilder::new().default_data()
                        .title("Guild Join").unwrap()
                        .description(format!("Name: {}\nServer Id: {}\nOwner Id: {}\nMembercount: {}", guild.name, guild.id.0, guild.owner_id.0, guild.member_count.unwrap_or_default())).unwrap()
                        .build().unwrap();
                    ctx.logger.log_event(&ctx, log_embed).await;
                }
            },
            Event::GuildDelete(guild) => {
                let log_embed = EmbedBuilder::new().default_data()
                    .title("Guild Leave").unwrap()
                    .description(format!("Server Id: {}", guild.id.0)).unwrap()
                    .build().unwrap();
                ctx.logger.log_event(&ctx, log_embed).await;
            }
            Event::Ready(ready) => {
                tracing::debug!("RoWifi ready for service!");
                for status in ready.guilds.values() {
                    if let GuildStatus::Offline(ug) = status {
                        self.0.unavailable.insert(ug.id);
                    }
                }
                let guild_ids = ready.guilds.keys().map(|k| k.0).collect::<Vec<u64>>();
                let guilds = ctx.database.get_guilds(&guild_ids, false).await?;
                for guild in guilds {
                    if let Some(command_prefix) = guild.command_prefix {
                        ctx.config.prefixes.insert(GuildId(guild.id as u64), command_prefix);
                        for channel in guild.disabled_channels {
                            ctx.config.disabled_channels.insert(ChannelId(channel as u64));
                        }
                    }
                }
            },
            Event::UnavailableGuild(g) => {
                self.0.unavailable.insert(g.id);
            },
            Event::MemberAdd(m) => {
                let server = match ctx.cache.guild(m.guild_id) {
                    Some(s) => s,
                    None => return Ok(())
                };
                let member = match ctx.cache.member(m.guild_id, m.user.id) {
                    Some(m) => m,
                    None => return Ok(())
                };
                let guild = match ctx.database.get_guild(m.guild_id.0).await? {
                    Some(g) => g,
                    None => return Ok(())
                };
                if !guild.settings.update_on_join {
                    return Ok(())
                }
                let user = match ctx.database.get_user(m.user.id.0).await? {
                    Some(u) => u,
                    None => return Ok(())
                };
                if server.owner_id == m.user.id {
                    return Ok(())
                }
                let guild_roles = ctx.cache.roles(m.guild_id);
                let (added_roles, removed_roles, disc_nick) = user.update(ctx.http.clone(), member, ctx.roblox.clone(), server, &guild, &guild_roles).await?;
                let log_embed = EmbedBuilder::new()
                    .title("Update On Join").unwrap()
                    .update_log(&added_roles, &removed_roles, &disc_nick)
                    .build().unwrap();
                ctx.logger.log_guild(&ctx, m.guild_id, log_embed).await;
            }
            _ => {}
        }
        Ok(())
    }
}