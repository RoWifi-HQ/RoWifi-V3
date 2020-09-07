use crate::framework::prelude::Context;
use std::error::Error;
use twilight_model::id::{GuildId, UserId};
use tokio::time::{interval, Duration};

pub async fn auto_detection(ctx: Context) {
    let mut interval = interval(Duration::from_secs(3 * 3600));
    loop {
        interval.tick().await;
        let _ = execute(&ctx).await;
    }
}

async fn execute(ctx: &Context) -> Result<(), Box<dyn Error>> {
    let servers = ctx.cache.guilds();
    let guilds = ctx.database.get_guilds(servers, true).await?;
    for guild in guilds {
        let guild_id = GuildId(guild.id as u64);
        let server = match ctx.cache.guild(guild_id) {
            Some(g) => g,
            None => continue
        };
        let members = ctx.cache.members(guild_id).into_iter().map(|m| m.0).collect();
        let users = ctx.database.get_users(members).await?;
        let bypass = ctx.cache.bypass_roles(guild_id);
        let guild_roles = ctx.cache.roles(guild_id);
        for user in users {
            if let Some(member) = ctx.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if let Some(bypass) = bypass.0 {
                    if member.roles.contains(&bypass) {continue;}
                }
                let _ = user.update(ctx.http.clone(), member, ctx.roblox.clone(), server.clone(), &guild, &guild_roles).await;
            }
        }
    }
    Ok(())
}