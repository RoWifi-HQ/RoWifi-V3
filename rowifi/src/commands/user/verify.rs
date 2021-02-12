use rand::{thread_rng, Rng};
use rowifi_framework::prelude::*;
use rowifi_models::user::{QueueUser, RoUser};
use std::time::Duration;
use tokio::time::timeout;
use twilight_model::gateway::payload::MessageCreate;

pub static VERIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["verify"],
    desc: Some("Command to link Roblox Account to Discord Account"),
    usage: Some("verify <Roblox Username> <Code/Game>"),
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("User"),
};

pub static REVERIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["reverify"],
    desc: Some("Command to change the linked Roblox Account"),
    usage: Some("reverify <Roblox Username> <Code/Game>"),
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("User"),
};

pub static VERIFY_COMMAND: Command = Command {
    fun: verify,
    options: &VERIFY_OPTIONS,
};

pub static REVERIFY_COMMAND: Command = Command {
    fun: reverify,
    options: &REVERIFY_OPTIONS,
};

static CODES: &[&str] = &[
    "cat",
    "dog",
    "sun",
    "rain",
    "snow",
    "alcazar",
    "dight",
    "night",
    "morning",
    "eyewater",
    "flaws",
    "physics",
    "chemistry",
    "history",
    "martlet",
    "nagware",
    "coffee",
    "tea",
    "red",
    "blue",
    "green",
    "orange",
    "pink",
];

#[command]
pub async fn verify(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult {
    if ctx.database.get_user(msg.author.id.0).await?.is_some() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("User Already Verified")
            .unwrap()
            .description(
                "To change your verified account, use `reverify`. To get your roles, use `update`",
            )
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    verify_common(ctx, msg, args, false).await
}

#[command]
pub async fn reverify(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult {
    if ctx.database.get_user(msg.author.id.0).await?.is_none() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("User Not Verified")
            .unwrap()
            .description("You are not verified. Please use `verify` to link your account")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    verify_common(ctx, msg, args, true).await
}

pub async fn verify_common(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'_>,
    verified: bool,
) -> CommandResult {
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::Red as u32)
        .unwrap()
        .title("Verification Process Failed")
        .unwrap();

    let roblox_username = match args.next() {
        Some(r) => r.to_owned(),
        None => await_reply("Enter your Roblox Username", ctx, msg).await?,
    };

    let roblox_id = match ctx.roblox.get_id_from_username(&roblox_username).await? {
        Some(r) => r,
        None => {
            let e = embed
                .description("Invalid Roblox Username. Please try again.")
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(e)
                .unwrap()
                .await;
            return Ok(());
        }
    };

    let option = match args.next() {
        Some(o) => o.to_owned(),
        None => {
            await_reply(
                "Enter the type of verification you wish to perform.\n Options: `Code`, `Game`",
                ctx,
                msg,
            )
            .await?
        }
    };

    if option.eq_ignore_ascii_case("Code") {
        let code1 = thread_rng().gen_range(0..CODES.len());
        let code2 = thread_rng().gen_range(0..CODES.len());
        let code3 = thread_rng().gen_range(0..CODES.len());
        let code = format!("{} {} {}", CODES[code1], CODES[code2], CODES[code3]);
        let e = EmbedBuilder::new()
            .default_data()
            .field(
                EmbedFieldBuilder::new(
                    "Verification Process",
                    "Enter the following code in your Roblox status/description.",
                )
                .unwrap(),
            )
            .field(EmbedFieldBuilder::new("Code", code.clone()).unwrap())
            .field(
                EmbedFieldBuilder::new("Next Steps", "After doing so, reply to me saying 'done'.")
                    .unwrap(),
            )
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(e)
            .unwrap()
            .await;

        let id = msg.author.id;
        let fut = ctx
            .standby
            .wait_for_message(msg.channel_id, move |event: &MessageCreate| {
                event.author.id == id
                    && (event.content.eq_ignore_ascii_case("done")
                        || event.content.eq_ignore_ascii_case("cancel"))
            });
        match timeout(Duration::from_secs(300), fut).await {
            Ok(Ok(m)) => {
                if m.content.eq_ignore_ascii_case("cancel") {
                    let e = embed
                        .description("Command has been cancelled")
                        .unwrap()
                        .build()
                        .unwrap();
                    let _ = ctx
                        .http
                        .create_message(msg.channel_id)
                        .embed(e)
                        .unwrap()
                        .await?;
                    return Ok(());
                }
            }
            _ => {
                let e = embed
                    .description("Command timed out. Please try again.")
                    .unwrap()
                    .build()
                    .unwrap();
                let _ = ctx
                    .http
                    .create_message(msg.channel_id)
                    .embed(e)
                    .unwrap()
                    .await;
                return Ok(());
            }
        }

        if !ctx.roblox.check_code(roblox_id, &code).await? {
            let e = embed
                .description(format!(
                    "{} was not found in your profile. Please try again.",
                    code
                ))
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(e)
                .unwrap()
                .await;
            return Ok(());
        }

        let user = RoUser {
            discord_id: msg.author.id.0 as i64,
            roblox_id,
        };
        let _ = ctx.database.add_user(user, verified).await;
        let e = embed.color(Color::DarkGreen as u32).unwrap().title("Verification Successful").unwrap()
            .description("To get your roles, run `update`. To change your linked Roblox Account, use `reverify`").unwrap()
            .build().unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(e)
            .unwrap()
            .await;
    } else if option.eq_ignore_ascii_case("Game") {
        let game_url = "https://www.roblox.com/games/5146847848/Verification-Center";
        let e = EmbedBuilder::new()
            .default_data()
            .title("Verification Process")
            .unwrap()
            .field(
                EmbedFieldBuilder::new(
                    "Further Steps",
                    format!(
                        "Please join the following game to verify yourself: [Click Here]({})",
                        game_url
                    ),
                )
                .unwrap(),
            )
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(e)
            .unwrap()
            .await?;
        let q_user = QueueUser {
            roblox_id,
            discord_id: msg.author.id.0 as i64,
            verified,
        };
        ctx.database.add_queue_user(q_user).await?;
    } else {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Verification Failed")
            .unwrap()
            .description("Invalid Option selected. Available Options: `Code`, `Game`")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    if let Some(guild) = ctx.database.get_guild(msg.guild_id.unwrap().0).await? {
        if guild.settings.update_on_verify {
            //update(ctx, msg, args).await?;
        }
    }
    Ok(())
}
