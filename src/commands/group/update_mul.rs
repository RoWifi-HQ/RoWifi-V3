use std::sync::atomic::Ordering;

use crate::framework::prelude::*;
use crate::models::guild::GuildType;
use twilight_gateway::Event;
use twilight_model::{gateway::payload::RequestGuildMembers, id::UserId};

pub static UPDATE_ALL_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: Some("update-multiple"),
    names: &["update-all"],
    desc: Some("Command to update all members in the server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Premium"),
};

pub static UPDATE_ALL_COMMAND: Command = Command {
    fun: update_all,
    options: &UPDATE_ALL_OPTIONS,
};

pub static UPDATE_ROLE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: Some("update-multiple"),
    names: &["update-role"],
    desc: Some("Command to update all members with a certain role"),
    usage: None,
    examples: &[],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: Some("Premium"),
};

pub static UPDATE_ROLE_COMMAND: Command = Command {
    fun: update_role,
    options: &UPDATE_ROLE_OPTIONS,
};

#[command]
pub async fn update_all(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;
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
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .content("Updating all members...")
        .unwrap()
        .await?;
    let server = ctx.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0)
        .collect::<Vec<_>>();
    if members.len() < (server.member_count.load(Ordering::SeqCst) / 2) as usize {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0 >> 22) % ctx.bot_config.total_shards;
        if ctx.cluster.command(shard_id, &req).await.is_err() {
            let _ = ctx.http.create_message(msg.channel_id).content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.").unwrap().await;
            return Ok(());
        }
        let _ = ctx
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
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
    }
    let users = ctx.database.get_users(members).await?;
    let guild_roles = ctx.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = msg.channel_id;
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.cache.member(guild_id, UserId(user.discord_id as u64)) {
                if let Some(bypass) = server.bypass_role {
                    if member.roles.contains(&bypass) {
                        continue;
                    }
                }
                tracing::trace!(id = user.discord_id, "Mass Update for member");
                let name = member.user.name.clone();
                if let Ok((added_roles, removed_roles, disc_nick)) = user
                    .update(
                        c.http.clone(),
                        member,
                        c.roblox.clone(),
                        server.clone(),
                        &guild,
                        &guild_roles,
                    )
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
                        c.logger.log_guild(&c, guild_id, log_embed).await;
                    }
                }
            }
        }
        let _ = c
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .await;
    });
    Ok(())
}

#[command]
pub async fn update_role(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;
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
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let role_str = match args.next() {
        Some(r) => r,
        None => return Ok(()),
    };
    let role_id = match parse_role(role_str) {
        Some(v) if server_roles.contains(&RoleId(v)) => RoleId(v),
        _ => {
            return Err(CommandError::ParseArgument(
                role_str.into(),
                "Role".into(),
                "Discord Role/Number".into(),
            )
            .into())
        }
    };

    let server = ctx.cache.guild(guild_id).unwrap();
    let mut members = ctx
        .cache
        .members(guild_id)
        .into_iter()
        .map(|m| m.0)
        .collect::<Vec<_>>();
    if members.len() < (server.member_count.load(Ordering::SeqCst) / 2) as usize {
        let req = RequestGuildMembers::builder(server.id).query("", None);
        let shard_id = (guild_id.0 >> 22) % ctx.bot_config.total_shards;
        if ctx.cluster.command(shard_id, &req).await.is_err() {
            let _ = ctx.http.create_message(msg.channel_id).content("There was an issue in requesting the server members. Please try again. If the issue persists, please contact our support server.").unwrap().await;
            return Ok(());
        }
        let _ = ctx
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
            .cache
            .members(guild_id)
            .into_iter()
            .map(|m| m.0)
            .collect::<Vec<_>>();
    }
    let users = ctx.database.get_users(members).await?;
    let guild_roles = ctx.cache.roles(guild_id);
    let c = ctx.clone();
    let channel_id = msg.channel_id;
    tokio::spawn(async move {
        for user in users {
            if let Some(member) = c.cache.member(guild_id, UserId(user.discord_id as u64)) {
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
                if let Ok((added_roles, removed_roles, disc_nick)) = user
                    .update(
                        c.http.clone(),
                        member,
                        c.roblox.clone(),
                        server.clone(),
                        &guild,
                        &guild_roles,
                    )
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
                        c.logger.log_guild(&c, guild_id, log_embed).await;
                    }
                }
            }
        }
        let _ = c
            .http
            .create_message(channel_id)
            .content("Finished updating all members")
            .unwrap()
            .await;
    });
    Ok(())
}
