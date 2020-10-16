use crate::{framework::prelude::Context, utils::misc::EmbedExtensions};
use std::{error::Error, sync::atomic::Ordering};
use tokio::time::{interval, Duration};
use twilight_embed_builder::EmbedBuilder;
use twilight_model::{
    gateway::payload::RequestGuildMembers,
    id::{GuildId, UserId},
};

pub async fn auto_detection(ctx: Context, total_shards: u64) {
    let mut interval = interval(Duration::from_secs(3 * 3600));
    std::thread::sleep(Duration::from_secs(60));
    loop {
        interval.tick().await;
        let _ = execute(&ctx, total_shards).await;
    }
}

async fn execute(ctx: &Context, total_shards: u64) -> Result<(), Box<dyn Error>> {
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
        let members = ctx
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
        if members.len() < (server.member_count.load(Ordering::SeqCst) / 2) as usize {
            let req = RequestGuildMembers::builder(server.id).query("", None);
            let shard_id = (guild_id.0 >> 22) % total_shards;
            let _res = ctx.cluster.command(shard_id, &req).await;
            continue;
        }
        let users = ctx.database.get_users(members).await?;
        let guild_roles = ctx.cache.roles(guild_id);
        for user in users {
            if let Some(member) = ctx.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {
                        continue;
                    }
                }
                tracing::trace!(id = user.discord_id, "Auto Detection for member");
                let name = member.user.name.clone();
                if let Ok((added_roles, removed_roles, disc_nick)) = user
                    .update(
                        ctx.http.clone(),
                        member,
                        ctx.roblox.clone(),
                        server.clone(),
                        &guild,
                        &guild_roles,
                    )
                    .await
                {
                    if !added_roles.is_empty() || !removed_roles.is_empty() {
                        let log_embed = EmbedBuilder::new()
                            .default_data()
                            .title(format!("Auto Detection: {}", name))
                            .unwrap()
                            .update_log(&added_roles, &removed_roles, &disc_nick)
                            .build()
                            .unwrap();
                        ctx.logger.log_guild(ctx, guild_id, log_embed).await;
                    }
                }
            }
        }
        let end = chrono::Utc::now().timestamp_millis();
        tracing::info!(time = end-start, server_name = ?server.name, "Time to complete auto detection");
        ctx.logger
            .log_premium(ctx, &format!("{} - {}", server.name, end - start))
            .await;
    }
    Ok(())
}
