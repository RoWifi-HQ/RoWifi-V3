use crate::{
    context::CommandContext,
    error::{CommandError, RoError},
};

use std::{
    cmp::{max, min},
    time::Duration,
};
use tokio::{stream::StreamExt, time::timeout};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};
use twilight_http::request::prelude::RequestReactionType;
use twilight_model::{
    channel::{embed::Embed, Message, ReactionType},
    gateway::payload::{MessageCreate, ReactionAdd},
    id::RoleId,
};

pub enum Color {
    Red = 0x00E7_4C3C,
    Blue = 0x0034_98DB,
    DarkGreen = 0x001F_8B4C,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
#[repr(i8)]
pub enum RoLevel {
    Creator = 3,
    Admin = 2,
    Trainer = 1,
    Normal = 0,
}

impl Default for RoLevel {
    fn default() -> Self {
        RoLevel::Normal
    }
}

pub async fn await_reply(
    question: &str,
    ctx: &CommandContext,
    msg: &Message,
) -> Result<String, RoError> {
    let question = format!("{}\nSay `cancel` to cancel this prompt", question);
    ctx.bot
        .http
        .create_message(msg.channel_id)
        .content(question)
        .unwrap()
        .await?;
    let id = msg.author.id;
    let fut = ctx
        .bot
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
    ctx: &CommandContext,
    msg: &Message,
    pages: Vec<Embed>,
    page_count: usize,
) -> Result<(), RoError> {
    let page_count = page_count as isize;
    if page_count <= 1 {
        let _ = ctx
            .bot
            .http
            .create_message(msg.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .await?;
    } else {
        let m = ctx
            .bot
            .http
            .create_message(msg.channel_id)
            .embed(pages[0].clone())
            .unwrap()
            .await?;

        //Get some easy named vars
        let channel_id = m.channel_id;
        let message_id = m.id;
        let author_id = msg.author.id;
        let http = ctx.bot.http.clone();

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
            .bot
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
                    .bot
                    .http
                    .update_message(channel_id, message_id)
                    .embed(pages[page_pointer as usize].clone())
                    .unwrap()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .delete_reaction(channel_id, message_id, react, author_id)
                    .await;
            }
        }
        let _ = ctx.bot.http.delete_message(channel_id, message_id).await;
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
            .map(|a| format!("- <@&{}>\n", a.0))
            .collect::<String>();
        let mut removed_str = removed_roles
            .iter()
            .map(|r| format!("- <@&{}>\n", r.0))
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
