use itertools::Itertools;
use rowifi_cache::CachedGuild;
use rowifi_database::postgres::Row;
use rowifi_framework::{context::BotContext, prelude::*};
use rowifi_models::{
    bind::Bind,
    discord::gateway::{event::Event, payload::outgoing::RequestGuildMembers},
    guild::{GuildType, RoGuild},
    id::{GuildId, RoleId},
    roblox::id::UserId as RobloxUserId,
    user::RoGuildUser,
};
use std::{collections::HashSet, env, error::Error, sync::atomic::Ordering};
use tokio::time::{interval, sleep, timeout, Duration};

use crate::utils::{UpdateUser, UpdateUserResult};

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
    let servers = ctx
        .cache
        .guilds()
        .into_iter()
        .map(|s| s as i64)
        .collect::<Vec<_>>();

    let mut guilds = ctx.database.query::<RoGuild>("SELECT * FROM guilds WHERE guild_id = ANY($1) AND (kind = $2 OR kind = $3) AND auto_detection = true", &[&servers, &GuildType::Alpha, &GuildType::Beta]).await?;
    guilds.sort_by_key(|g| g.guild_id);
    for guild in guilds {
        let start = chrono::Utc::now().timestamp_millis();
        let guild_id = guild.guild_id;
        let server = match ctx.cache.guild(guild_id) {
            Some(g) => g,
            None => continue,
        };
        let binds = ctx
            .database
            .query::<Bind>(
                "SELECT * FROM binds WHERE guild_id = $1",
                &[&guild.guild_id],
            )
            .await?;
        let mut members = ctx
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0.get() as i64)
            .collect::<Vec<_>>();
        if (members.len() as i64) < server.member_count.load(Ordering::SeqCst) / 2 {
            let req = RequestGuildMembers::builder(server.id.0).query("", None);
            let shard_id = (guild_id.0.get() >> 22) % ctx.total_shards;
            let fut = timeout(
                Duration::from_secs(30),
                ctx.standby.wait_for_event(move |event: &Event| {
                    if let Event::MemberChunk(mc) = event {
                        if mc.guild_id == guild_id.0 && mc.chunk_index == mc.chunk_count - 1 {
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
                .map(|m| m.0.get() as i64)
                .collect::<Vec<_>>();
        }
        let rows = ctx
            .database
            .query::<Row>(
                r#"
                SELECT users.discord_id, l.roblox_id, users.default_roblox_id FROM 
                (SELECT * FROM linked_users WHERE guild_id = $1) AS l
                RIGHT JOIN users
                ON users.discord_id = l.discord_id
                WHERE users.discord_id = ANY($2)
            "#,
                &[&guild.guild_id, &members],
            )
            .await?;
        let mut users = Vec::new();
        for row in rows {
            match mass_update_user(&row, guild.guild_id) {
                Ok(u) => users.push(u),
                Err(err) => tracing::error!("error in deserializing user: {}", err),
            }
        }
        tracing::trace!("got users: {:?}", users);
        let guild_roles = ctx.cache.roles(guild_id);
        let all_roles = binds
            .iter()
            .flat_map(|b| b.discord_roles())
            .unique()
            .collect::<Vec<_>>();
        for user_chunk in users.chunks(100) {
            let user_ids = user_chunk
                .iter()
                .map(|u| RobloxUserId(u.roblox_id as u64))
                .collect_vec();
            if let Err(err) = ctx.roblox.get_users(&user_ids).await {
                tracing::error!(err = ?err);
            }
            for user_sec_chunk in user_chunk.chunks(chunk_size) {
                let (_, _) = tokio::join!(
                    execute_chunk(
                        user_sec_chunk,
                        ctx,
                        &server,
                        &guild,
                        &guild_roles,
                        true,
                        None,
                        &binds,
                        &all_roles
                    ),
                    sleep(Duration::from_secs(1))
                );
            }
        }
        let end = chrono::Utc::now().timestamp_millis();
        tracing::info!(time = end-start, server_name = ?server.name, "Time to complete auto detection");
        ctx.log_premium(&format!("{} - {}", server.name, end - start))
            .await;
    }
    tracing::info!("Auto Detection Cycle complete");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_chunk(
    user_chunk: &[RoGuildUser],
    ctx: &BotContext,
    server: &CachedGuild,
    guild: &RoGuild,
    guild_roles: &HashSet<RoleId>,
    auto_detection: bool,
    role_filter: Option<RoleId>,
    binds: &[Bind],
    all_roles: &[&RoleId],
) -> Result<(), RoError> {
    let log = if auto_detection {
        "Auto Detection"
    } else {
        "Mass Update"
    };
    for user in user_chunk {
        if let Some(member) = ctx.cache.member(server.id, user.discord_id) {
            if let Some(role_filter) = role_filter {
                if !member.roles.contains(&role_filter) {
                    continue;
                }
            }
            if ctx.has_bypass_role(server, &member) {
                continue;
            }
            tracing::trace!("{} for user id: {}", log, user.discord_id);
            let name = member.user.name.clone();

            let update_user = UpdateUser {
                ctx,
                member: &member,
                user,
                server,
                guild,
                binds,
                guild_roles,
                bypass_roblox_cache: false,
                all_roles,
            };

            let res = update_user.execute().await;
            if let UpdateUserResult::Success(added_roles, removed_roles, disc_nick) = res {
                if !added_roles.is_empty() || !removed_roles.is_empty() {
                    let log_embed = EmbedBuilder::new()
                        .default_data()
                        .title(format!("{}: {}", log, name))
                        .update_log(&added_roles, &removed_roles, &disc_nick)
                        .build();
                    ctx.log_guild(server.id, log_embed).await;
                }
            } else if let UpdateUserResult::Error(err) = res {
                tracing::error!(err = ?err);
            }
        }
    }
    Ok(())
}

pub fn mass_update_user(row: &Row, guild_id: GuildId) -> Result<RoGuildUser, Box<dyn Error>> {
    let guild_id = row.try_get("guild_id").unwrap_or(guild_id);
    let discord_id = row.try_get("discord_id")?;
    let roblox_id = row.try_get("roblox_id").ok();
    let default_roblox_id = row.try_get("default_roblox_id")?;

    Ok(RoGuildUser {
        guild_id,
        discord_id,
        roblox_id: roblox_id.unwrap_or(default_roblox_id),
    })
}
