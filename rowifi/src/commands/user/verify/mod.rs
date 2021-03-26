mod manage;

use rowifi_framework::prelude::*;
use rowifi_models::user::QueueUser;

use manage::{verify_default, verify_delete, verify_switch};

pub fn verify_config(cmds: &mut Vec<Command>) {
    let verify_add_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["add"])
        .description("Command to link an additional Roblox account to your Discord Account")
        .handler(verify_add);

    let verify_switch_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["switch"])
        .description("Command to switch your Roblox account for this server")
        .handler(verify_switch);

    let verify_default_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["default"])
        .description("Command to change the default Roblox Account")
        .handler(verify_default);

    let verify_delete_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["delete"])
        .description("Command to unlink a non-default account")
        .handler(verify_delete);

    let verify_view_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["view"])
        .description("Command to view all linked accounts")
        .handler(verify_view);

    let verify_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["verify"])
        .description("Command to link a roblox account to your discord account")
        .group("User")
        .sub_command(verify_add_cmd)
        .sub_command(verify_switch_cmd)
        .sub_command(verify_default_cmd)
        .sub_command(verify_delete_cmd)
        .sub_command(verify_view_cmd)
        .handler(verify);

    cmds.push(verify_cmd);
}

#[derive(FromArgs)]
pub struct VerifyArguments {
    #[arg(help = "The Roblox Username to verify to")]
    pub username: Option<String>,
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

pub async fn verify_add(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
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
    Ok(())
}

#[derive(FromArgs)]
pub struct VerifyViewArguments {}

pub async fn verify_view(ctx: CommandContext, _args: VerifyViewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let user = match ctx.bot.database.get_user(ctx.author.id.0).await? {
        Some(u) => u,
        None => {
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
    };
    let linked_user = ctx
        .bot
        .database
        .get_linked_user(ctx.author.id.0, guild_id.0)
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Linked Accounts")
        .unwrap()
        .color(Color::Blue as u32)
        .unwrap();

    let mut acc_string = String::new();

    let main_username = ctx.bot.roblox.get_username_from_id(user.roblox_id).await?;
    acc_string.push_str(&main_username);
    acc_string.push_str(" - `Default`");
    if let Some(linked_user) = &linked_user {
        if linked_user.roblox_id == user.roblox_id {
            acc_string.push_str(", `This Server`");
        }
    } else {
        acc_string.push_str(", `This Server`");
    }
    acc_string.push('\n');
    for alt in &user.alts {
        let username = ctx.bot.roblox.get_username_from_id(*alt).await?;
        acc_string.push_str(&username);
        if let Some(linked_user) = &linked_user {
            if linked_user.roblox_id == *alt {
                acc_string.push_str(" - `This Server`");
            }
        }
        acc_string.push('\n');
    }

    let embed = embed.description(acc_string).unwrap().build().unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    Ok(())
}
