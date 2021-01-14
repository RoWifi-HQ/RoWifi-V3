use framework_new::prelude::*;
use reqwest::StatusCode;
use twilight_embed_builder::EmbedFooterBuilder;
use twilight_http::Error as DiscordHttpError;
use twilight_model::id::{RoleId, UserId};

#[derive(Debug, FromArgs)]
pub struct UpdateArguments {
    pub user_id: Option<UserId>,
}

pub async fn update(ctx: CommandContext, args: UpdateArguments) -> Result<(), RoError> {
    let start = chrono::Utc::now().timestamp_millis();
    let guild_id = ctx.guild_id.unwrap();
    let server = ctx.bot.cache.guild(guild_id).unwrap();

    let user_id = match args.user_id {
        Some(s) => s,
        None => ctx.author_id,
    };

    let member = match ctx.member(guild_id, user_id).await? {
        Some(m) => m,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .unwrap()
                .description("No such member was found")
                .unwrap()
                .color(Color::Red as u32)
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await;
            return Ok(());
        }
    };

    //Check for server owner
    if server.owner_id.0 == member.user.id.0 {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Update Failed")
            .unwrap()
            .description("Due to discord limitations, I cannot update the server owner")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await;
        return Ok(());
    }

    //Handle role position check

    //Check for bypass role
    if let Some(bypass_role) = &server.bypass_role {
        if member.roles.contains(bypass_role) {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .unwrap()
                .description("I cannot update users with the `RoWifi Bypass` role")
                .unwrap()
                .color(Color::Red as u32)
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await;
            return Ok(());
        }
    }

    let user = match ctx.bot.database.get_user(user_id.0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .unwrap()
                .description("User was not verified. Please ask him/her to verify themselves")
                .unwrap()
                .color(Color::Red as u32)
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await;
            return Ok(());
        }
    };

    let guild = match ctx.bot.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .unwrap()
                .description(
                    "Server is not set up. Please ask the server owner to set up the server.",
                )
                .unwrap()
                .color(Color::Red as u32)
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await;
            return Ok(());
        }
    };
    let guild_roles = ctx.bot.cache.roles(guild_id);

    let (added_roles, removed_roles, disc_nick): (Vec<RoleId>, Vec<RoleId>, String) = match ctx
        .update_user(member, &user, server, &guild, &guild_roles)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            if let RoError::Discord(DiscordHttpError::Response {
                body: _,
                error: _,
                status,
            }) = e
            {
                if status == StatusCode::FORBIDDEN {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::Red as u32)
                        .unwrap()
                        .title("Update Failed")
                        .unwrap()
                        .description("You seem to have a role higher than or equal to mine")
                        .unwrap()
                        .build()
                        .unwrap();
                    let _ = ctx
                        .bot
                        .http
                        .create_message(ctx.channel_id)
                        .embed(embed)
                        .unwrap()
                        .await;
                    return Ok(());
                }
            } else if let RoError::Command(CommandError::Blacklist(ref b)) = e {
                if let Ok(channel) = ctx.bot.http.create_private_channel(user_id).await {
                    let _ = ctx
                        .bot
                        .http
                        .create_message(channel.id)
                        .content(format!(
                            "You were found on the server blacklist. Reason: {}",
                            b
                        ))
                        .unwrap()
                        .await;
                }
            }
            return Err(e);
        }
    };
    let end = chrono::Utc::now().timestamp_millis();
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Update")
        .unwrap()
        .update_log(&added_roles, &removed_roles, &disc_nick)
        .color(Color::DarkGreen as u32)
        .unwrap()
        .footer(
            EmbedFooterBuilder::new(format!("RoWifi | Executed in {} ms", (end - start))).unwrap(),
        )
        .build()
        .unwrap();
    let _ = ctx
        .bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await;

    // let log_embed = EmbedBuilder::new()
    //     .default_data()
    //     .title("Update")
    //     .unwrap()
    //     .update_log(&added_roles, &removed_roles, &disc_nick)
    //     .build()
    //     .unwrap();
    //ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
