use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;
use std::sync::atomic::Ordering;
use twilight_gateway::Event;
use twilight_model::{
    gateway::payload::RequestGuildMembers,
    id::{RoleId, UserId},
};

#[derive(FromArgs)]
pub struct UpdateAllArguments {}

pub async fn update_all(ctx: CommandContext, _args: UpdateAllArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Update All Failed")
            .unwrap()
            .description("This command may only be used in Premium Servers")
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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .content("Updating all members...")
        .unwrap()
        .await?;
    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .bot
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0)
        .collect::<Vec<_>>();
    if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0 >> 22) % ctx.bot.total_shards;
        if ctx.bot.cluster.command(shard_id, &req).await.is_err() {
            ctx.bot.http.create_message(ctx.channel_id).content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.").unwrap().await?;
            return Ok(());
        }
        let _ = ctx
            .bot
            .standby
            .wait_for_event(move |event: &Event| {
                if let Event::MemberChunk(mc) = event {
                    if mc.guild_id == guild_id && mc.chunk_index == mc.chunk_count - 1 {
                        return true;
                    }
                }
                false
            })
            .await;
        members = ctx
            .bot
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
    }
    let users = ctx
        .bot
        .database
        .get_linked_users(&members, guild_id.0)
        .await?;
    let guild_roles = ctx.bot.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = ctx.channel_id;
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.bot.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {
                        continue;
                    }
                }
                tracing::trace!(id = user.discord_id, "Mass Update for member");
                let name = member.user.name.clone();
                if let Ok((added_roles, removed_roles, disc_nick)) = c
                    .update_user(member, &user, &server, &guild, &guild_roles)
                    .await
                {
                    if !added_roles.is_empty() || !removed_roles.is_empty() {
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title(format!("Mass Update: {}", name))
                            .unwrap()
                            .update_log(&added_roles, &removed_roles, &disc_nick)
                            .build()
                            .unwrap();
                        c.log_guild(guild_id, log_embed).await;
                    }
                }
            }
        }
        let _ = c
            .bot
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .await;
    });
    Ok(())
}

#[derive(FromArgs)]
pub struct UpdateMultipleArguments {
    #[arg(help = "The role or its id whose members are to be updated")]
    pub role: RoleId,
}

pub async fn update_role(ctx: CommandContext, args: UpdateMultipleArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Update All Failed")
            .unwrap()
            .description("This command may only be used in Premium Servers")
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

    let server_roles = ctx.bot.cache.roles(guild_id);
    let role_id = args.role;
    if !server_roles.contains(&role_id) {
        return Err(RoError::Argument(ArgumentError::ParseError {
            expected: "a Discord Role/Id",
            usage: UpdateMultipleArguments::generate_help(),
            name: "role",
        }));
    }

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .bot
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0)
        .collect::<Vec<_>>();
    if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0 >> 22) % ctx.bot.total_shards;
        if ctx.bot.cluster.command(shard_id, &req).await.is_err() {
            ctx.bot.http.create_message(ctx.channel_id).content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.").unwrap().await?;
            return Ok(());
        }
        let _ = ctx
            .bot
            .standby
            .wait_for_event(move |event: &Event| {
                if let Event::MemberChunk(mc) = event {
                    if mc.guild_id == guild_id && mc.chunk_index == mc.chunk_count - 1 {
                        return true;
                    }
                }
                false
            })
            .await;
        members = ctx
            .bot
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
    }
    let users = ctx
        .bot
        .database
        .get_linked_users(&members, guild_id.0)
        .await?;
    let guild_roles = ctx.bot.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = ctx.channel_id;
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.bot.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if !member.roles.contains(&role_id) {
                    continue;
                }
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {
                        continue;
                    }
                }
                tracing::trace!(id = user.discord_id, "Mass Update for member");
                let name = member.user.name.clone();
                if let Ok((added_roles, removed_roles, disc_nick)) = c
                    .update_user(member, &user, &server, &guild, &guild_roles)
                    .await
                {
                    if !added_roles.is_empty() || !removed_roles.is_empty() {
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title(format!("Mass Update: {}", name))
                            .unwrap()
                            .update_log(&added_roles, &removed_roles, &disc_nick)
                            .build()
                            .unwrap();
                        c.log_guild(guild_id, log_embed).await;
                    }
                }
            }
        }
        let _ = c
            .bot
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .await;
    });
    Ok(())
}
