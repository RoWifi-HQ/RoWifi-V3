use crate::framework::prelude::Context;
use super::error::{RoError, CommandError};
use std::{time::Duration, cmp::{max, min}};
use twilight_http::request::prelude::RequestReactionType;
use twilight_model::{
    channel::{Message, embed::Embed}, 
    gateway::payload::{MessageCreate, ReactionAdd},
    channel::ReactionType
};
use tokio::{time::timeout, stream::StreamExt};

pub async fn await_reply(question: &str, ctx: &Context, msg: &Message) -> Result<String, RoError> {
    let question = format!("{}\nSay `cancel` to cancel this prompt", question);
    let _ = ctx.http.create_message(msg.channel_id).content(question).unwrap().await?;
    let id = msg.author.id;
    let fut = ctx.standby.wait_for_message(msg.channel_id, move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
    match timeout(Duration::from_secs(300), fut).await {
        Ok(Ok(m)) if !m.content.eq_ignore_ascii_case("cancel") => Ok(m.content.to_owned()),
        _ => Err(RoError::Command(CommandError::Timeout))
    }
}

pub async fn paginate_embed(ctx: &Context, msg: &Message, pages: Vec<Embed>, page_count: usize) -> Result<(), RoError> {
    if page_count <= 1 {
        let _ = ctx.http.create_message(msg.channel_id).embed(pages[0].clone()).unwrap().await?;
    } else {
        let m = ctx.http.create_message(msg.channel_id).embed(pages[0].clone()).unwrap().await?;

        //Get some easy named vars
        let channel_id = m.channel_id;
        let message_id = m.id;
        let author_id = msg.author.id;
        let http = ctx.http.clone();

        //Don't wait up for the reactions to show
        tokio::spawn(async move {
            let _ = http.create_reaction(channel_id, message_id, RequestReactionType::Unicode {name: String::from("⏮️") }).await;
            let _ = http.create_reaction(channel_id, message_id, RequestReactionType::Unicode {name: String::from("◀️") }).await;
            let _ = http.create_reaction(channel_id, message_id, RequestReactionType::Unicode {name: String::from("▶️") }).await;
            let _ = http.create_reaction(channel_id, message_id, RequestReactionType::Unicode {name: String::from("⏭️") }).await;
            let _ = http.create_reaction(channel_id, message_id, RequestReactionType::Unicode {name: String::from("⏹️") }).await;
        });

        let mut reactions = ctx.standby.wait_for_reaction_stream(message_id, move |event: &ReactionAdd| {
            if event.user_id != author_id {
                return false;
            }
            if let ReactionType::Unicode{name} = &event.emoji {
                return matches!(&name[..], "⏮️" | "◀️" | "▶️" | "⏭️" | "⏹️")
            }
           false
        }).timeout(Duration::from_secs(60));

        let mut page_pointer: usize = 0;
        while let Some(Ok(reaction)) = reactions.next().await {
            if let ReactionType::Unicode{name} = &reaction.emoji {
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
                let react = RequestReactionType::Unicode {name: name.clone()};
                let _ = ctx.http.update_message(channel_id, message_id).embed(pages[page_pointer].clone()).unwrap().await;
                let _ = ctx.http.delete_reaction(channel_id, message_id, react, author_id).await;
            }
        }
        let _ = ctx.http.delete_message(channel_id, message_id).await;
    }
    Ok(())
}