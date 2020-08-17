use crate::framework::prelude::*;
use rand::{thread_rng, Rng};
use std::time::Duration;
use tokio::time::timeout;
use twilight::model::gateway::payload::MessageCreate;
use twilight_embed_builder::EmbedFieldBuilder;

use crate::models::user::RoUser;

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

pub static VERIFY_COMMAND: Command = Command {
    fun: verify,
    options: &VERIFY_OPTIONS
};

static CODES: &'static [&'static str] = &["cat", "dog", "sun", "rain", "snow", "alcazar", "dight", "night", "morning", 
    "eyewater", "flaws", "physics", "chemistry", "history", "martlet", "nagware", "coffee", "tea", "red", "blue", "green", 
    "orange", "pink"
];

#[command]
pub async fn verify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult { 
    // match ctx.database.get_user(msg.author.id.0).await? {
    //     None => {},
    //     Some(_) => {
    //         // Handle error
    //         return  Ok(())
    //     }
    // }

    let roblox_username = match args.next() {
        Some(r) => r.to_owned(),
        None => {
            let _ = ctx.http.as_ref().create_message(msg.channel_id).content("Enter your roblox username").unwrap().await;
            let id = msg.author.id;
            let fut = ctx.standby.wait_for_message(msg.channel_id, 
                move |event: &MessageCreate| event.author.id == id && event.content.len() > 0);
            match timeout(Duration::from_secs(300), fut).await {
                Ok(Ok(m)) => m.0.content,
                _ => {
                    //Handle this shit
                    return Ok(())
                }
            }
        }
    };
    
    let roblox_id = match ctx.roblox.get_id_from_username(&roblox_username).await? {
        Some(r) => r,
        None => {
            println!("Bad Username");
            return Ok(())
        }
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => {
            let _ = ctx.http.as_ref().create_message(msg.channel_id).content("Enter Option").unwrap().await;
            let id = msg.author.id;
            let fut = ctx.standby.wait_for_message(msg.channel_id, 
            move |event: &MessageCreate| event.author.id == id && 
                        (event.content.eq_ignore_ascii_case("Code") || event.content.eq_ignore_ascii_case("Game")));
            
            match timeout(Duration::from_secs(300), fut).await {
                Ok(Ok(m)) => m.0.content,
                _ => {
                    //Handle Error
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
        let embed = EmbedBuilder::new()
            .default_data()
            .field(EmbedFieldBuilder::new("Verification Process", "Do it later").unwrap())
            .field(EmbedFieldBuilder::new("Code", code.clone()).unwrap())
            .field(EmbedFieldBuilder::new("Next Steps", "Do it later").unwrap())
            .build().unwrap();
        let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;

        let id = msg.author.id;
        let fut = ctx.standby.wait_for_message(msg.channel_id, 
            move |event: &MessageCreate| event.author.id == id && event.content.eq_ignore_ascii_case("done"));
        if let Err(_) = timeout(Duration::from_secs(300), fut).await {
            //Handle Error
            return Ok(())
        }

        if !ctx.roblox.as_ref().check_code(roblox_id, &code).await? {
            //Give Error
            return Ok(())
        }

        let user = RoUser { discord_id: msg.author.id.0 as i64, roblox_id };
        let _ = ctx.database.as_ref().add_user(user, true).await;
        //Send Embed
    }

    Ok(())
}