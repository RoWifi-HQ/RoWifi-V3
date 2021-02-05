use framework_new::prelude::*;
use rand::{thread_rng, Rng};
use rowifi_models::user::{QueueUser, RoUser};
use std::time::Duration;
use tokio::time::timeout;
use twilight_model::gateway::payload::MessageCreate;

use super::update::{update, UpdateArguments};

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

#[derive(FromArgs)]
pub struct VerifyArguments {
    #[arg(help = "The Roblox Username to verify to")]
    pub username: Option<String>,
    #[arg(help = "The verification option type")]
    pub option: Option<VerifyOption>,
}

pub enum VerifyOption {
    Game,
    Code,
}

pub async fn verify(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    if ctx.bot.database.get_user(ctx.author.id.0).await?.is_some() {
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    verify_common(ctx, args, false).await
}

pub async fn reverify(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    if ctx.bot.database.get_user(ctx.author.id.0).await?.is_none() {
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    verify_common(ctx, args, true).await
}

pub async fn verify_common(
    ctx: CommandContext,
    args: VerifyArguments,
    verified: bool,
) -> CommandResult {
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::Red as u32)
        .unwrap()
        .title("Verification Process Failed")
        .unwrap();

    let roblox_username = match args.username {
        Some(r) => r,
        None => await_reply("Enter your Roblox Username", &ctx).await?,
    };

    let roblox_id = match ctx
        .bot
        .roblox
        .get_id_from_username(&roblox_username)
        .await?
    {
        Some(r) => r,
        None => {
            let e = embed
                .description("Invalid Roblox Username. Please try again.")
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(e)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let option = match args.option {
        Some(o) => o,
        None => {
            let ans = await_reply(
                "Enter the type of verification you wish to perform.\n Options: `Code`, `Game`",
                &ctx,
            )
            .await?;
            match VerifyOption::from_arg(&ans) {
                Ok(o) => o,
                Err(_) => {
                    ctx.bot
                        .http
                        .create_message(ctx.channel_id)
                        .content("Invalid Option Selected. Avaliable Options: `Code` `Game`")
                        .unwrap()
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    match option {
        VerifyOption::Code => {
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
                    EmbedFieldBuilder::new(
                        "Next Steps",
                        "After doing so, reply to me saying 'done'.",
                    )
                    .unwrap(),
                )
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(e)
                .unwrap()
                .await?;

            let id = ctx.author.id;
            let fut =
                ctx.bot
                    .standby
                    .wait_for_message(ctx.channel_id, move |event: &MessageCreate| {
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
                        ctx.bot
                            .http
                            .create_message(ctx.channel_id)
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
                    ctx.bot
                        .http
                        .create_message(ctx.channel_id)
                        .embed(e)
                        .unwrap()
                        .await?;
                    return Ok(());
                }
            }

            if !ctx.bot.roblox.check_code(roblox_id, &code).await? {
                let e = embed
                    .description(format!(
                        "{} was not found in your profile. Please try again.",
                        code
                    ))
                    .unwrap()
                    .build()
                    .unwrap();
                ctx.bot
                    .http
                    .create_message(ctx.channel_id)
                    .embed(e)
                    .unwrap()
                    .await?;
                return Ok(());
            }

            let user = RoUser {
                discord_id: ctx.author.id.0 as i64,
                roblox_id,
            };
            ctx.bot.database.add_user(user, verified).await?;
            let e = embed.color(Color::DarkGreen as u32).unwrap().title("Verification Successful").unwrap()
                .description("To get your roles, run `update`. To change your linked Roblox Account, use `reverify`").unwrap()
                .build().unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(e)
                .unwrap()
                .await?;
        }
        VerifyOption::Game => {
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(e)
                .unwrap()
                .await?;
            let q_user = QueueUser {
                roblox_id,
                discord_id: ctx.author.id.0 as i64,
                verified,
            };
            ctx.bot.database.add_queue_user(q_user).await?;
        }
    };

    if let Some(guild_id) = ctx.guild_id {
        if let Some(guild) = ctx.bot.database.get_guild(guild_id.0).await? {
            if guild.settings.update_on_verify {
                let args = UpdateArguments { user_id: None };
                update(ctx, args).await?;
            }
        }
    }
    Ok(())
}

impl FromArg for VerifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "game" => Ok(VerifyOption::Game),
            "code" => Ok(VerifyOption::Code),
            _ => Err(ParseError("one of `game` `code`")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("VerifyOption unreached"),
        };

        Self::from_arg(&arg)
    }
}
