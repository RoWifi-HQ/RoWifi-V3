use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::{gateway::payload::outgoing::RequestGuildMembers, id::RoleId},
    guild::GuildType,
    roblox::id::UserId as RobloxUserId,
};
use std::{env, sync::atomic::Ordering};
use tokio::time::sleep;
use twilight_gateway::Event;

use crate::services::auto_detection::execute_chunk;

pub async fn update_all(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Update All Failed")
            .description("This command may only be used in Premium Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    ctx.respond()
        .content("Updating all members...")?
        .exec()
        .await?;
    tracing::info!("Update-all queue started in {}", guild_id);

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Started an `update-all` queue")
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .bot
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0.get() as i64)
        .collect::<Vec<_>>();
    if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0.get() >> 22) % ctx.bot.total_shards;
        if ctx.bot.cluster.command(shard_id, &req).await.is_err() {
            ctx.respond().content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.")?.exec().await?;
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
            .map(|m| m.0.get() as i64)
            .collect::<Vec<_>>();
    }
    let users = ctx
        .bot
        .database
        .get_linked_users(&members, guild_id.0.get())
        .await?;
    let guild_roles = ctx.bot.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = ctx.channel_id;

    let chunk_size = if let Ok(chunk_size) = env::var("CHUNK_SIZE") {
        chunk_size.parse::<usize>().unwrap_or(5)
    } else {
        5
    };

    tokio::spawn(async move {
        for user_chunk in users.chunks(100) {
            let user_ids = user_chunk
                .iter()
                .map(|u| RobloxUserId(u.roblox_id as u64))
                .collect::<Vec<_>>();
            if let Err(err) = c.bot.roblox.get_users(&user_ids).await {
                tracing::error!(err = ?err);
            }
            for user_sec_chunk in user_chunk.chunks(chunk_size) {
                let (_, _) = tokio::join!(
                    execute_chunk(
                        user_sec_chunk,
                        &ctx.bot,
                        &server,
                        &guild,
                        &guild_roles,
                        false,
                        None
                    ),
                    sleep(Duration::from_secs(1))
                );
            }
        }
        let _ = c
            .bot
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .exec()
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
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;
    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Update All Failed")
            .description("This command may only be used in Premium Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let server_roles = ctx.bot.cache.roles(guild_id);
    let role_id = args.role;
    if !server_roles.contains(&role_id) {
        return Err(ArgumentError::ParseError {
            expected: "a Discord Role/Id",
            usage: UpdateMultipleArguments::generate_help(),
            name: "role",
        }
        .into());
    }

    ctx.respond()
        .content("Updating all members...")?
        .exec()
        .await?;
    tracing::info!("Update-all queue started in {}", guild_id);

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Started an `update-role` queue")
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let server = ctx.bot.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .bot
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0.get() as i64)
        .collect::<Vec<_>>();
    if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0.get() >> 22) % ctx.bot.total_shards;
        if ctx.bot.cluster.command(shard_id, &req).await.is_err() {
            ctx.respond().content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.")?.exec().await?;
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
            .map(|m| m.0.get() as i64)
            .collect::<Vec<_>>();
    }
    let users = ctx
        .bot
        .database
        .get_linked_users(&members, guild_id.0.get())
        .await?;
    let guild_roles = ctx.bot.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = ctx.channel_id;

    let chunk_size = if let Ok(chunk_size) = env::var("CHUNK_SIZE") {
        chunk_size.parse::<usize>().unwrap_or(5)
    } else {
        5
    };

    tokio::spawn(async move {
        for user_chunk in users.chunks(100) {
            let user_ids = user_chunk
                .iter()
                .map(|u| RobloxUserId(u.roblox_id as u64))
                .collect::<Vec<_>>();
            if let Err(err) = c.bot.roblox.get_users(&user_ids).await {
                tracing::error!(err = ?err);
            }
            for user_sec_chunk in user_chunk.chunks(chunk_size) {
                let (_, _) = tokio::join!(
                    execute_chunk(
                        user_sec_chunk,
                        &ctx.bot,
                        &server,
                        &guild,
                        &guild_roles,
                        false,
                        Some(role_id)
                    ),
                    sleep(Duration::from_secs(1))
                );
            }
        }
        let _ = c
            .bot
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .exec()
            .await;
    });
    Ok(())
}
