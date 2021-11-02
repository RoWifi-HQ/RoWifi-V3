mod manage;

use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::application::component::{
        action_row::ActionRow,
        button::{Button, ButtonStyle},
        Component,
    },
    roblox::id::UserId as RobloxUserId,
    user::QueueUser,
};

use crate::commands::handle_update_button;

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

    let verify_setup_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["setup"])
        .description("Command to link a roblox account to your discord account")
        .handler(verify);

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
        .sub_command(verify_setup_cmd)
        .handler(verify);

    cmds.push(verify_cmd);
}

#[derive(FromArgs)]
pub struct VerifyArguments {
    #[arg(help = "The Roblox Username to verify to")]
    pub username: Option<String>,
}

pub async fn verify(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    if ctx
        .bot
        .database
        .get_user(ctx.author.id.0.get())
        .await?
        .is_some()
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("User Already Verified")
            .description(
                "To link another account, use `verify add`. To get your roles, use `update`. To switch your account on this server, you must use `verify switch`. To set a default account on new servers, you must use `verify default`.",
            )
            .color(Color::Red as u32)
            .build()
            .unwrap();
        let message = ctx
            .respond()
            .embeds(&[embed])?
            .components(&[Component::ActionRow(ActionRow {
                components: vec![Component::Button(Button {
                    custom_id: Some("handle-update".into()),
                    disabled: false,
                    emoji: None,
                    label: Some("Update your Roles".into()),
                    style: ButtonStyle::Primary,
                    url: None,
                })],
            })])?
            .exec()
            .await?
            .model()
            .await?;

        handle_update_button(&ctx, message.id, Vec::new()).await?;
        return Ok(());
    }
    verify_common(ctx, args, false).await
}

pub async fn verify_add(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    if ctx
        .bot
        .database
        .get_user(ctx.author.id.0.get())
        .await?
        .is_none()
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("User Not Verified")
            .description("You are not verified. Please use `verify` to link your account")
            .color(Color::Red as u32)
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
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
        .title("Verification Process Failed");

    let roblox_username = match args.username {
        Some(r) => r,
        None => await_reply("Enter your Roblox Username", &ctx).await?,
    };

    let roblox_id = match ctx
        .bot
        .roblox
        .get_user_from_username(&roblox_username)
        .await?
    {
        Some(r) => r,
        None => {
            let embed = embed
                .description("Invalid Roblox Username. Please try again.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let roblox_id = roblox_id.id.0 as i64;

    let game_url = "https://www.roblox.com/games/5146847848/Verification-Center";
    let e = EmbedBuilder::new()
        .default_data()
        .title("Verification Process")
        .field(
            EmbedFieldBuilder::new(
                "Further Steps",
                "Join the game below to continue the verification process"
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "Post Verification", 
                "Once successfully verified, you must use `update` to get your roles. To switch your account on this server, you must use `verify switch`. To set a default account on new servers, you must use `verify default`."
            )
        )
        .build()
        .unwrap();

    let game_url_button = Component::Button(Button {
        style: ButtonStyle::Link,
        emoji: None,
        label: Some("Join the Game".into()),
        custom_id: None,
        url: Some(game_url.into()),
        disabled: false,
    });
    let message = ctx
        .respond()
        .embeds(&[e])?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![
                game_url_button.clone(),
                Component::Button(Button {
                    custom_id: Some("handle-update".into()),
                    disabled: false,
                    emoji: None,
                    label: Some("Update your Roles".into()),
                    style: ButtonStyle::Primary,
                    url: None,
                }),
            ],
        })])?
        .exec()
        .await?
        .model()
        .await?;
    let q_user = QueueUser {
        roblox_id,
        discord_id: ctx.author.id.0.get() as i64,
        verified,
    };
    ctx.bot.database.add_queue_user(q_user).await?;

    handle_update_button(
        &ctx,
        message.id,
        vec![Component::ActionRow(ActionRow {
            components: vec![game_url_button],
        })],
    )
    .await?;
    Ok(())
}

pub async fn verify_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let user = match ctx.bot.database.get_user(ctx.author.id.0.get()).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("User Not Verified")
                .description("You are not verified. Please use `verify` to link your account")
                .color(Color::Red as u32)
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let linked_user = ctx
        .bot
        .database
        .get_linked_user(ctx.author.id.0.get(), guild_id.0.get())
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Linked Accounts")
        .color(Color::Blue as u32);

    let mut acc_string = String::new();

    let main_user = ctx
        .bot
        .roblox
        .get_user(RobloxUserId(user.roblox_id as u64), false)
        .await?;
    acc_string.push_str(&main_user.name);
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
        let user = ctx
            .bot
            .roblox
            .get_user(RobloxUserId(*alt as u64), false)
            .await?;
        acc_string.push_str(&user.name);
        if let Some(linked_user) = &linked_user {
            if linked_user.roblox_id == *alt {
                acc_string.push_str(" - `This Server`");
            }
        }
        acc_string.push('\n');
    }

    let embed = embed.description(acc_string).build().unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
