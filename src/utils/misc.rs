use super::error::{CommandError, RoError};
use crate::cache::{CachedGuild, CachedRole};
use crate::framework::prelude::Context;
use std::{
    cmp::{max, min},
    collections::HashMap,
    sync::Arc,
    time::Duration,
};
use tokio::{stream::StreamExt, time::timeout};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};
use twilight_http::request::prelude::RequestReactionType;
use twilight_mention::Mention;
use twilight_model::{
    channel::{
        embed::Embed, permission_overwrite::PermissionOverwriteType, GuildChannel, Message,
        ReactionType,
    },
    gateway::payload::{MessageCreate, ReactionAdd},
    guild::Permissions,
    id::{RoleId, UserId},
};

pub enum Color {
    Red = 0xE74C3C,
    Blue = 0x3498DB,
    DarkGreen = 0x1F8B4C,
}

pub async fn await_reply(question: &str, ctx: &Context, msg: &Message) -> Result<String, RoError> {
    let question = format!("{}\nSay `cancel` to cancel this prompt", question);
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .content(question)
        .unwrap()
        .await?;
    let id = msg.author.id;
    let fut = ctx
        .standby
        .wait_for_message(msg.channel_id, move |event: &MessageCreate| {
            event.author.id == id && !event.content.is_empty()
        });
    match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => Ok(m.content.to_owned()),
        _ => Err(RoError::Command(CommandError::Timeout)),
    }
}

pub async fn paginate_embed(
    ctx: &Context,
    msg: &Message,
    pages: Vec<Embed>,
    page_count: usize,
) -> Result<(), RoError> {
    let page_count = page_count as isize;
    if page_count <= 1 {
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .await?;
    } else {
        let m = ctx
            .http
            .create_message(msg.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .await?;

        //Get some easy named vars
        let channel_id = m.channel_id;
        let message_id = m.id;
        let author_id = msg.author.id;
        let http = ctx.http.clone();

        //Don't wait up for the reactions to show
        tokio::spawn(async move {
            let _ = http
                .create_reaction(
                    channel_id,
                    message_id,
                    RequestReactionType::Unicode {
                        name: String::from("⏮️"),
                    },
                )
                .await;
            let _ = http
                .create_reaction(
                    channel_id,
                    message_id,
                    RequestReactionType::Unicode {
                        name: String::from("◀️"),
                    },
                )
                .await;
            let _ = http
                .create_reaction(
                    channel_id,
                    message_id,
                    RequestReactionType::Unicode {
                        name: String::from("▶️"),
                    },
                )
                .await;
            let _ = http
                .create_reaction(
                    channel_id,
                    message_id,
                    RequestReactionType::Unicode {
                        name: String::from("⏭️"),
                    },
                )
                .await;
            let _ = http
                .create_reaction(
                    channel_id,
                    message_id,
                    RequestReactionType::Unicode {
                        name: String::from("⏹️"),
                    },
                )
                .await;
        });

        let mut reactions = ctx
            .standby
            .wait_for_reaction_stream(message_id, move |event: &ReactionAdd| {
                if event.user_id != author_id {
                    return false;
                }
                if let ReactionType::Unicode { name } = &event.emoji {
                    return matches!(&name[..], "⏮️" | "◀️" | "▶️" | "⏭️" | "⏹️");
                }
                false
            })
            .timeout(Duration::from_secs(300));

        let mut page_pointer: isize = 0;
        while let Some(Ok(reaction)) = reactions.next().await {
            if let ReactionType::Unicode { name } = &reaction.emoji {
                if name == "⏮️" {
                    page_pointer = 0;
                } else if name == "◀️" {
                    page_pointer = max(page_pointer - 1, 0);
                } else if name == "▶️" {
                    page_pointer = min(page_pointer + 1, page_count - 1);
                } else if name == "⏭️" {
                    page_pointer = page_count - 1;
                } else if name == "⏹️" {
                    break;
                }
                let react = RequestReactionType::Unicode { name: name.clone() };
                let _ = ctx
                    .http
                    .update_message(channel_id, message_id)
                    .embed(pages[page_pointer as usize].clone())
                    .unwrap()
                    .await;
                let _ = ctx
                    .http
                    .delete_reaction(channel_id, message_id, react, author_id)
                    .await;
            }
        }
        let _ = ctx.http.delete_message(channel_id, message_id).await;
    }
    Ok(())
}

pub fn parse_username(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@!") {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if mention.starts_with("<@") {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    }
}

pub fn parse_role(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    }
}

pub trait EmbedExtensions {
    fn default_data(self) -> Self;
    fn update_log(self, added_roles: &[RoleId], removed_roles: &[RoleId], disc_nick: &str) -> Self;
}

impl EmbedExtensions for EmbedBuilder {
    fn default_data(self) -> Self {
        self.timestamp(&chrono::Utc::now().to_rfc3339())
            .color(Color::Blue as u32)
            .expect("Some shit occurred with the embed color")
            .footer(
                EmbedFooterBuilder::new("RoWifi").expect("Looks like the footer text screwed up"),
            )
    }

    fn update_log(self, added_roles: &[RoleId], removed_roles: &[RoleId], disc_nick: &str) -> Self {
        let mut added_str = added_roles
            .iter()
            .map(|a| format!("- {}\n", a.mention()))
            .collect::<String>();
        let mut removed_str = removed_roles
            .iter()
            .map(|r| format!("- {}\n", r.mention()))
            .collect::<String>();
        if added_str.is_empty() {
            added_str = "None".into();
        }
        if removed_str.is_empty() {
            removed_str = "None".into();
        }

        self.field(EmbedFieldBuilder::new("Nickname", disc_nick).unwrap())
            .field(EmbedFieldBuilder::new("Added Roles", added_str).unwrap())
            .field(EmbedFieldBuilder::new("Removed Roles", removed_str).unwrap())
    }
}

pub fn guild_wide_permissions(
    guild: Arc<CachedGuild>,
    roles: &HashMap<RoleId, Arc<CachedRole>>,
    member_id: UserId,
    member_roles: &[RoleId],
) -> Result<Permissions, String> {
    if member_id == guild.owner_id {
        return Ok(Permissions::all());
    }

    let mut permissions = match roles.get(&RoleId(guild.id.0)) {
        Some(r) => r.permissions,
        None => return Err("`@everyone` role is missing from the cache.".into()),
    };

    for role in member_roles {
        let role_permissions = match roles.get(&role) {
            Some(r) => r.permissions,
            None => return Err("Found a role on the member that doesn't exist on the cache".into()),
        };

        permissions |= role_permissions;
    }
    Ok(permissions)
}

pub fn channel_permissions(
    guild: Arc<CachedGuild>,
    roles: &HashMap<RoleId, Arc<CachedRole>>,
    member_id: UserId,
    member_roles: &[RoleId],
    channel: Arc<GuildChannel>,
) -> Result<Permissions, String> {
    let guild_id = guild.id;
    let mut permissions = guild_wide_permissions(guild, roles, member_id, &member_roles)?;
    let mut member_allow = Permissions::empty();
    let mut member_deny = Permissions::empty();
    let mut roles_allow = Permissions::empty();
    let mut roles_deny = Permissions::empty();

    if let GuildChannel::Text(tc) = channel.as_ref() {
        for overwrite in tc.permission_overwrites.iter() {
            match overwrite.kind {
                PermissionOverwriteType::Role(role) => {
                    if role.0 == guild_id.0 {
                        permissions.remove(overwrite.deny);
                        permissions.insert(overwrite.allow);
                        continue;
                    }

                    if !member_roles.contains(&role) {
                        continue;
                    }

                    roles_allow.insert(overwrite.allow);
                    roles_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(user) if user == member_id => {
                    member_allow.insert(overwrite.allow);
                    member_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(_) => {}
            }
        }
        permissions.remove(roles_deny);
        permissions.insert(roles_allow);
        permissions.remove(member_deny);
        permissions.insert(member_allow);

        return Ok(permissions);
    }

    Err("Not implemented for non text guild channels".into())
}
