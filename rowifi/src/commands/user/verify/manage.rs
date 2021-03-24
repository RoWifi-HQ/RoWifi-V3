use mongodb::bson::oid::ObjectId;
use rowifi_framework::prelude::*;
use rowifi_models::user::RoGuildUser;

use super::VerifyArguments;

pub async fn verify_switch(ctx: CommandContext, args: VerifyArguments) -> CommandResult {
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
            let e = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Verification Process Failed")
                .unwrap()
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

    if !user.alts.contains(&roblox_id) {
        let e = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Verification Process Failed")
            .unwrap()
            .description("The provided username is not linked to your discord account. Please link it using `verify add`")
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

    let linked_user = RoGuildUser {
        id: ObjectId::new(),
        guild_id: guild_id.0 as i64,
        user_id: ctx.author.id.0 as i64,
        roblox_id,
    };

    ctx.bot.database.add_linked_user(linked_user).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Account Switching Successful")
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    Ok(())
}

pub async fn verify_default(_ctx: CommandContext, _args: VerifyArguments) -> CommandResult {
    Ok(())
}
