use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Rankbind},
    id::RoleId,
    roblox::id::GroupId,
};

use super::new::PREFIX_REGEX;

#[derive(FromArgs)]
pub struct ModifyPriority {
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The priority to set")]
    pub priority: i32,
}

pub async fn rb_modify_priority(ctx: CommandContext, args: ModifyPriority) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    let group_id = args.group_id;
    let rank_id = args.rank_id;
    let priority = args.priority;

    let bind = match rankbinds
        .iter()
        .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Rank Bind Modification Failed")
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    ctx.bot
        .database
        .execute(
            "UPDATE binds SET priority = $1 WHERE bind_id = $2",
            &[&priority, &bind.bind_id],
        )
        .await?;

    let name = format!("Group Id: {group_id}");
    let desc = format!("Rank Id: {rank_id}\n`Priority`: {0} -> {priority}", bind.priority);

    rb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct ModifyTemplate {
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The template to set", rest)]
    pub template: String,
}

pub async fn rb_modify_template(ctx: CommandContext, args: ModifyTemplate) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    let rank_id = args.rank_id;
    let template = args.template;

    if template.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Rank Bind Modification Failed")
            .description("You have entered a blank template")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    let bind = match rankbinds
        .iter()
        .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Rank Bind Modification Failed")
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let roblox_group = ctx
        .bot
        .roblox
        .get_group_ranks(GroupId(group_id as u64))
        .await?;
    let roblox_rank = match &roblox_group {
        Some(g) => g.roles.iter().find(|r| i64::from(r.rank) == rank_id),
        None => None,
    };
    let template = match template.as_str() {
        "auto" => {
            if let Some(rank) = roblox_rank {
                if let Some(m) = PREFIX_REGEX.captures(&rank.name) {
                    format!("[{}] {{roblox-username}}", m.get(1).unwrap().as_str())
                } else {
                    "{roblox-username}".into()
                }
            } else {
                "{roblox-username}".into()
            }
        }
        "disable" => "{discord-name}".into(),
        "N/A" => "{roblox-username}".into(),
        _ => template.to_string(),
    };
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET template = $1 WHERE bind_id = $2",
            &[&template, &bind.bind_id],
        )
        .await?;

    let name = format!("Group Id: {group_id}");
    let desc = format!("Rank Id: {rank_id}\n`Template`: {0} -> {template}", bind.template);

    rb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct AddRoles {
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The roles to add", rest)]
    pub roles: String,
}

pub async fn rb_add_roles(ctx: CommandContext, args: AddRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    let rank_id = args.rank_id;
    
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    let bind = match rankbinds
        .iter()
        .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Rank Bind Modification Failed")
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    ctx.bot.database.execute("UPDATE binds SET discord_roles = array_cat(discord_roles, $1::BIGINT[]) WHERE bind_id = $2", &[&role_ids, &bind.bind_id]).await?;

    let modification = role_ids
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let name = format!("Group Id: {group_id}");
    let desc = format!("Rank Id: {rank_id}\n`Added Roles`: {modification}");
    
    rb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct RemoveRoles {
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The roles to remove", rest)]
    pub roles: String,
}

pub async fn rb_remove_roles(ctx: CommandContext, args: RemoveRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    let rank_id = args.rank_id;
    
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    let bind = match rankbinds
        .iter()
        .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Rank Bind Modification Failed")
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let mut roles_to_keep = bind.discord_roles.clone();
    roles_to_keep.retain(|r| !role_ids.contains(r));
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET discord_roles = $1 WHERE bind_id = $2",
            &[&roles_to_keep, &bind.bind_id],
        )
        .await?;

    let modification = role_ids
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let name = format!("Group Id: {group_id}");
    let desc = format!("Rank Id: {rank_id}\n`Removed Roles`: {modification}");

    rb_reply_log(ctx, name, desc).await
}

async fn rb_reply_log(ctx: CommandContext, name: String, desc: String) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Success!")
        .description("The bind was successfully modified")
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Rank Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(())
}