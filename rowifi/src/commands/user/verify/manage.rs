use rowifi_framework::prelude::*;
use rowifi_models::user::RoGuildUser;

use crate::commands::handle_update_button;

use super::VerifyArguments;

pub async fn verify_switch(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
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
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Verification Process Failed")
                .description("Invalid Roblox Username. Please try again.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let roblox_id = roblox_id.id.0 as i64;

    if !user.alts.contains(&roblox_id) && user.roblox_id != roblox_id {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Verification Process Failed")
            .description("The provided username is not linked to your discord account. Please link it using `verify add`")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let linked_user = RoGuildUser {
        guild_id: guild_id.0.get() as i64,
        discord_id: ctx.author.id.0.get() as i64,
        roblox_id,
    };

    ctx.bot.database.add_linked_user(linked_user).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Account Switching Successful")
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

    Ok(())
}

pub async fn verify_default(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    let mut user = match ctx.bot.database.get_user(ctx.author.id.0.get()).await? {
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
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Verification Process Failed")
                .description("Invalid Roblox Username. Please try again.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let roblox_id = roblox_id.id.0 as i64;

    let account_index = match user.alts.iter().position(|r| *r == roblox_id) {
        Some(i) => i,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Verification Process Failed")
                .description("The provided username is not linked to your discord account or is already your default account")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    user.alts.remove(account_index);
    user.alts.push(user.roblox_id);
    user.roblox_id = roblox_id;
    ctx.bot.database.add_user(user, true).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Default Account Set Successfully")
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

    Ok(())
}

pub async fn verify_delete(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
    let mut user = match ctx.bot.database.get_user(ctx.author.id.0.get()).await? {
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
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Verification Process Failed")
                .description("Invalid Roblox Username. Please try again.")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let roblox_id = roblox_id.id.0 as i64;

    let account_index = match user.alts.iter().position(|r| *r == roblox_id) {
        Some(i) => i,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Verification Process Failed")
                .description("The provided username is not linked to your discord account or is your default account")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    user.alts.remove(account_index);
    ctx.bot
        .database
        .delete_linked_users(ctx.author.id.0.get(), roblox_id)
        .await?;
    ctx.bot.database.add_user(user, true).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Account Unlinking Successful")
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

    Ok(())
}
