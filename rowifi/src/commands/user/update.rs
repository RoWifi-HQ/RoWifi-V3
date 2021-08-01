use hyper::StatusCode;
use rowifi_framework::prelude::*;
use twilight_embed_builder::EmbedFooterBuilder;
use twilight_http::error::ErrorType as DiscordErrorType;
use twilight_model::{
    channel::embed::Embed,
    id::{RoleId, UserId},
};

#[derive(Debug, FromArgs)]
pub struct UpdateArguments {
    #[arg(help = "The user to be updated")]
    pub user_id: Option<UserId>,
}

pub async fn update(ctx: CommandContext, args: UpdateArguments) -> Result<(), RoError> {
    let embed = update_func(&ctx, args).await?;
    ctx.respond().embed(embed).await?;
    Ok(())
}

pub async fn update_func(ctx: &CommandContext, args: UpdateArguments) -> Result<Embed, RoError> {
    let start = chrono::Utc::now().timestamp_millis();
    let guild_id = ctx.guild_id.unwrap();
    let server = ctx.bot.cache.guild(guild_id).unwrap();

    let user_id = match args.user_id {
        Some(s) => s,
        None => ctx.author.id,
    };

    let member = match ctx.member(guild_id, user_id).await? {
        Some(m) => m,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .description("No such member was found")
                .color(Color::Red as u32)
                .build()
                .unwrap();
            return Ok(embed);
        }
    };

    //Check for server owner
    if server.owner_id.0 == member.user.id.0 {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Update Failed")
            .description("Due to discord limitations, I cannot update the server owner")
            .color(Color::Red as u32)
            .build()
            .unwrap();
        return Ok(embed);
    }

    //Handle role position check

    //Check for bypass role
    if ctx.bot.has_bypass_role(&server, &member) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Update Failed")
            .description("I cannot update users with roles having the `RoWifi Bypass` permission")
            .color(Color::Red as u32)
            .build()
            .unwrap();
        return Ok(embed);
    }

    let user = match ctx.get_linked_user(user_id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed")
                .description("User was not verified. Please ask them to verify themselves")
                .color(Color::Red as u32)
                .build()
                .unwrap();
            return Ok(embed);
        }
    };

    let guild = ctx.bot.database.get_guild(guild_id.0).await?;
    let guild_roles = ctx.bot.cache.roles(guild_id);

    let (added_roles, removed_roles, disc_nick): (Vec<RoleId>, Vec<RoleId>, String) = match ctx
        .update_user(member, &user, &server, &guild, &guild_roles)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            if let RoError::Discord(d) = &e {
                if let DiscordErrorType::Response {
                    body: _,
                    error: _,
                    status,
                } = d.kind()
                {
                    if *status == StatusCode::FORBIDDEN {
                        let embed = EmbedBuilder::new()
                            .default_data()
                            .color(Color::Red as u32)
                            .title("Update Failed")
                            .description(
                                "There was an error in updating the user. Possible causes:
                            1. The user has a role higher than or equal to mine
                            2. I am trying to add/remove a binded role that is above my highest role
                            3. Either the verification & verified role are above my highest role",
                            )
                            .build()
                            .unwrap();
                        return Ok(embed);
                    }
                }
            } else if let RoError::Command(CommandError::Blacklist(ref b)) = e {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Update Failed")
                    .description(format!(
                        "User was found on the server blacklist. Reason: {}",
                        b
                    ))
                    .build()
                    .unwrap();
                if let Ok(channel) = ctx.bot.http.create_private_channel(user_id).await {
                    let _ = ctx
                        .bot
                        .http
                        .create_message(channel.id)
                        .content(format!(
                            "You were found on the {} blacklist. Reason: {}",
                            server.name, b
                        ))
                        .unwrap()
                        .await;
                }
                return Ok(embed);
            }
            return Err(e);
        }
    };
    let end = chrono::Utc::now().timestamp_millis();
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Update")
        .update_log(&added_roles, &removed_roles, &disc_nick)
        .color(Color::DarkGreen as u32)
        .footer(EmbedFooterBuilder::new(format!(
            "RoWifi | Executed in {} ms",
            (end - start)
        )))
        .build()
        .unwrap();

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title("Update")
        .update_log(&added_roles, &removed_roles, &disc_nick)
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(embed)
}
