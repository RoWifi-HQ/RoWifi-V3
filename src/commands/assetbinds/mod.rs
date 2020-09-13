mod new;
mod modify;
mod delete;

use crate::framework::prelude::*;
use itertools::Itertools;
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_mention::Mention;
use twilight_model::{gateway::payload::ReactionAdd, channel::ReactionType};
use std::{cmp::{min, max}, time::Duration};
use tokio::stream::StreamExt;

pub use new::*;
pub use modify::*;
pub use delete::*;

pub static ASSETBINDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["assetbinds", "ab"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&ASSETBINDS_NEW_COMMAND, &ASSETBINDS_MODIFY_COMMAND, &ASSETBINDS_DELETE_COMMAND],
    group: Some("Binds")
};

pub static ASSETBINDS_COMMAND: Command = Command {
    fun: assetbind,
    options: &ASSETBINDS_OPTIONS
};

#[command]
pub async fn assetbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    if guild.assetbinds.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No assetbinds were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for binds in guild.assetbinds.iter().chunks(12).into_iter() {
        let mut embed = EmbedBuilder::new().default_data().title("AssetBinds").unwrap()
                .description(format!("Page {}", page_count+1)).unwrap();
        for ab in binds {
            let name = format!("Id: {}", ab.id);
            let roles_str = ab.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>();
            let desc = format!("Type: {}\nRoles: {}", ab.asset_type, roles_str);
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }

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
            let _ = http.create_reaction(channel_id, message_id, ReactionType::Unicode {name: String::from("⏮️") }).await;
            let _ = http.create_reaction(channel_id, message_id, ReactionType::Unicode {name: String::from("◀️") }).await;
            let _ = http.create_reaction(channel_id, message_id, ReactionType::Unicode {name: String::from("▶️") }).await;
            let _ = http.create_reaction(channel_id, message_id, ReactionType::Unicode {name: String::from("⏭️") }).await;
            let _ = http.create_reaction(channel_id, message_id, ReactionType::Unicode {name: String::from("⏹️") }).await;
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
                let _ = ctx.http.update_message(channel_id, message_id).embed(pages[page_pointer].clone()).unwrap().await;
                let _ = ctx.http.delete_reaction(channel_id, message_id, reaction.emoji.clone(), author_id).await;
            }
        }
        let _ = ctx.http.delete_message(channel_id, message_id).await;
    }

    Ok(())
}