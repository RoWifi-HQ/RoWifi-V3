use itertools::Itertools;
use rowifi_cache::CachedGuild;
use rowifi_framework::{
    context::BotContext,
    prelude::{EmbedExtensions, RoError},
};
use rowifi_models::{guild::RoGuild, roblox::id::UserId as RobloxUserId, user::RoGuildUser};
use std::{collections::HashSet, env, error::Error, sync::atomic::Ordering};
use tokio::time::{interval, sleep, timeout, Duration};
use twilight_embed_builder::EmbedBuilder;
use twilight_model::{
    gateway::{event::Event, payload::RequestGuildMembers},
    id::{GuildId, RoleId, UserId},
};

pub async fn auto_detection(ctx: BotContext) {
    tracing::info!("Auto Detection starting");
    let mut interval = interval(Duration::from_secs(3 * 3600));
    let chunk_size = if let Ok(chunk_size) = env::var("CHUNK_SIZE") {
        chunk_size.parse::<usize>().unwrap_or(5)
    } else {
        5
    };
    loop {
        interval.tick().await;
        if let Err(err) = execute(&ctx, chunk_size).await {
            tracing::error!(err = ?err, "Error in auto detection");
        }
    }
}

async fn execute(ctx: &BotContext, chunk_size: usize) -> Result<(), Box<dyn Error>> {
    let servers = ctx.cache.guilds();
    let mut guilds = ctx.database.get_guilds(&servers, true).await?;
    guilds.sort_by_key(|g| g.id);
    for guild in guilds {
        let start = chrono::Utc::now().timestamp_millis();
        let guild_id = GuildId(guild.id as u64);
        let server = match ctx.cache.guild(guild_id) {
            Some(g) => g,
            None => continue,
        };
        let mut members = ctx
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
        if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
            let req = RequestGuildMembers::builder(server.id).query("", None);
            let shard_id = (guild_id.0 >> 22) % ctx.total_shards;
            let fut = timeout(
                Duration::from_secs(30),
                ctx.standby.wait_for_event(move |event: &Event| {
                    if let Event::MemberChunk(mc) = event {
                        if mc.guild_id == guild_id && mc.chunk_index == mc.chunk_count - 1 {
                            return true;
                        }
                    }
                    false
                }),
            );
            ctx.cluster.command(shard_id, &req).await?;
            let _ = fut.await;
            members = ctx
                .cache
                .members(guild_id)
                .into_iter()
                .map(|m| m.0)
                .collect::<Vec<_>>();
        }
        let users = ctx.database.get_linked_users(&members, guild_id.0).await?;
        let guild_roles = ctx.cache.roles(guild_id);
        for user_chunk in users.chunks(50) {
            let user_ids = user_chunk
                .iter()
                .map(|u| RobloxUserId(u.roblox_id as u64))
                .collect_vec();
            let _roblox_ids = ctx.roblox.get_users(&user_ids).await;
            for user_sec_chunk in user_chunk.chunks(chunk_size) {
                let (_, _) = tokio::join!(
                    execute_chunk(user_sec_chunk, ctx, &server, &guild, &guild_roles),
                    sleep(Duration::from_secs(1))
                );
            }
        }
        let end = chrono::Utc::now().timestamp_millis();
        tracing::info!(time = end-start, server_name = ?server.name, "Time to complete auto detection");
        ctx.log_premium(&format!("{} - {}", server.name, end - start))
            .await;
    }
    Ok(())
}

async fn execute_chunk(
    user_chunk: &[RoGuildUser],
    ctx: &BotContext,
    server: &CachedGuild,
    guild: &RoGuild,
    guild_roles: &HashSet<RoleId>,
) -> Result<(), RoError> {
    for user in user_chunk {
        if let Some(member) = ctx.cache.member(server.id, UserId(user.discord_id as u64)) {
            if let Some(bypass) = server.bypass_role {
                if member.roles.contains(&bypass) {
                    continue;
                }
            }
            tracing::trace!(id = user.discord_id, "Auto Detection for member");
            let name = member.user.name.clone();
            if let Ok((added_roles, removed_roles, disc_nick)) = ctx
                .update_user(member, user, server, guild, guild_roles)
                .await
            {
                if !added_roles.is_empty() || !removed_roles.is_empty() {
                    let log_embed = EmbedBuilder::new()
                        .default_data()
                        .title(format!("Auto Detection: {}", name))
                        .update_log(&added_roles, &removed_roles, &disc_nick)
                        .build()
                        .unwrap();
                    ctx.log_guild(server.id, log_embed).await;
                }
            }
        }
    }
    Ok(())
}
