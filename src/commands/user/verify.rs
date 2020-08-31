use crate::framework::prelude::*;
use rand::{thread_rng, Rng};
use std::time::Duration;
use tokio::time::timeout;
use twilight_model::gateway::payload::MessageCreate;
use twilight_embed_builder::EmbedFieldBuilder;

use crate::models::user::{RoUser, QueueUser};

pub static VERIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["verify"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static REVERIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["reverify"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static VERIFY_COMMAND: Command = Command {
    fun: verify,
    options: &VERIFY_OPTIONS
};

pub static REVERIFY_COMMAND: Command = Command {
    fun: reverify,
    options: &REVERIFY_OPTIONS
};

static CODES: &[&str] = &["cat", "dog", "sun", "rain", "snow", "alcazar", "dight", "night", "morning", 
    "eyewater", "flaws", "physics", "chemistry", "history", "martlet", "nagware", "coffee", "tea", "red", "blue", "green", 
    "orange", "pink"
];

#[command]
pub async fn verify(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult { 
    if ctx.database.get_user(msg.author.id.0).await?.is_some() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("User Already Verified").unwrap()
            .description("To change your verified account, use `reverify`. To get your roles, use `update`").unwrap()
            .color(Color::Red as u32).unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;
        return Ok(())
    }
    verify_common(ctx, msg, args, false).await
}

#[command]
pub async fn reverify(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult { 
    if ctx.database.get_user(msg.author.id.0).await?.is_none() {
        //Give Error
        return Ok(())
    }
    verify_common(ctx, msg, args, true).await
}

pub async fn verify_common(ctx: &Context, msg: &Message, mut args: Arguments<'_>, verified: bool) -> CommandResult { 
    let embed = EmbedBuilder::new()
                .default_data().color(Color::Red as u32).unwrap()
                .title("Verification Process Failed").unwrap();

    let roblox_username = match args.next() {
        Some(r) => r.to_owned(),
        None => {
            let _ = ctx.http.create_message(msg.channel_id).content("Enter your Roblox Name.\nSay `cancel` if you wish to cancel this command").unwrap().await;

            let id = msg.author.id;
            let fut = ctx.standby.wait_for_message(msg.channel_id, 
                move |event: &MessageCreate| event.author.id == id && !event.content.is_empty());
            match timeout(Duration::from_secs(300), fut).await {
                Ok(Ok(m)) => {
                    if m.content.eq_ignore_ascii_case("cancel") {
                        let e = embed.description("Command has been cancelled").unwrap().build().unwrap();
                        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
                        return Ok(())
                    }
                    m.content.to_owned()
                },
                _ => {
                    let e = embed.description("Command timed out. Please try again.").unwrap().build().unwrap();
                    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
                    return Ok(())
                }
            }
        }
    };
    
    let roblox_id = match ctx.roblox.get_id_from_username(&roblox_username).await? {
        Some(r) => r,
        None => {
            let e = embed.description("Invalid Roblox Username. Please try again.").unwrap().build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => {
            let _ = ctx.http.create_message(msg.channel_id).content("Enter Option").unwrap().await;
            let id = msg.author.id;
            let fut = ctx.standby.wait_for_message(msg.channel_id, 
            move |event: &MessageCreate| event.author.id == id && 
                        (event.content.eq_ignore_ascii_case("Code") || event.content.eq_ignore_ascii_case("Game") || event.content.eq_ignore_ascii_case("cancel")));
            
            match timeout(Duration::from_secs(300), fut).await {
                Ok(Ok(m)) => {
                    if m.content.eq_ignore_ascii_case("cancel") {
                        let e = embed.description("Command has been cancelled").unwrap().build().unwrap();
                        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
                        return Ok(())
                    }
                    m.content.to_owned()
                },
                _ => {
                    let e = embed.description("Command timed out. Please try again.").unwrap().build().unwrap();
                    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
                    return Ok(())
                }
            }
        }
    };

    if option.eq_ignore_ascii_case("Code") {
        let code1 = thread_rng().gen_range(0, CODES.len());
        let code2 = thread_rng().gen_range(0, CODES.len());
        let code3 = thread_rng().gen_range(0, CODES.len());
        let code = format!("{} {} {}", CODES[code1], CODES[code2], CODES[code3]);
        let e = EmbedBuilder::new()
            .default_data()
            .field(EmbedFieldBuilder::new("Verification Process", "Enter the following code in your Roblox status/description.").unwrap())
            .field(EmbedFieldBuilder::new("Code", code.clone()).unwrap())
            .field(EmbedFieldBuilder::new("Next Steps", "After doing so, reply to me saying 'done'.").unwrap())
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;

        let id = msg.author.id;
        let fut = ctx.standby.wait_for_message(msg.channel_id, 
            move |event: &MessageCreate| event.author.id == id && (event.content.eq_ignore_ascii_case("done") || event.content.eq_ignore_ascii_case("cancel")));
        match timeout(Duration::from_secs(300), fut).await {
            Ok(Ok(m)) => {
                if m.content.eq_ignore_ascii_case("cancel") {
                    let e = embed.description("Command has been cancelled").unwrap().build().unwrap();
                    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
                    return Ok(())
                }
            }
            _ => {
                let e = embed.description("Command timed out. Please try again.").unwrap().build().unwrap();
                let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
                return Ok(())
            }
        }

        if !ctx.roblox.check_code(roblox_id, &code).await? {
            let e = embed.description(format!("{} was not found in your profile. Please try again.", code)).unwrap().build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
            return Ok(())
        }

        let user = RoUser { discord_id: msg.author.id.0 as i64, roblox_id };
        let _ = ctx.database.add_user(user, verified).await;
        let e = embed.color(Color::DarkGreen as u32).unwrap().title("Verification Successful").unwrap()
            .description("To get your roles, run `update`. To change your linked Roblox Account, use `reverify`").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
    } else if option.eq_ignore_ascii_case("Game") {
        let game_url = "https://www.roblox.com/games/5146847848/Verification-Center";
        let e = EmbedBuilder::new().default_data().title("Verification Process").unwrap()
            .field(EmbedFieldBuilder::new("Further Steps", format!("Please join the following game to verify yourself: [Click Here]({})", game_url)).unwrap())
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
        let q_user = QueueUser {roblox_id, discord_id: msg.author.id.0 as i64, verified};
        let _ = ctx.database.add_queue_user(q_user).await?;
    }

    Ok(())
}