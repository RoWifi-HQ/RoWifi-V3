use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Custombind},
    id::{RoleId, UserId},
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

#[derive(FromArgs)]
pub struct ModifyCode {
    #[arg(help = "The ID of the bind")]
    pub id: i32,
    #[arg(help = "The code to change to", rest)]
    pub code: String,
}

pub async fn cb_modify_code(ctx: CommandContext, args: ModifyCode) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let id_to_modify = args.id;
    let code = args.code;

    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let user = match ctx
        .bot
        .database
        .get_linked_user(UserId(ctx.author.id), guild_id)
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description("You must be verified to create a custombind")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx.member(guild_id, UserId(ctx.author.id)).await?.unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id, false).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(&s)?.exec().await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(&res)?.exec().await?;
        return Ok(());
    }
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET code = $1 WHERE bind_id = $2",
            &[&code, &bind.bind_id],
        )
        .await?;

    let name = format!("Id: {id_to_modify}");
    let desc = format!("`New Code`: {code}");

    cb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct ModifyPriority {
    #[arg(help = "The ID of the bind")]
    pub id: i32,
    #[arg(help = "The priority to change to")]
    pub priority: i32,
}

pub async fn cb_modify_priority(ctx: CommandContext, args: ModifyPriority) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let id_to_modify = args.id;
    let priority = args.priority;

    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
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

    let name = format!("Id: {id_to_modify}");
    let desc = format!("`Priority`: {0} -> {priority}", bind.priority);

    cb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct ModifyTemplate {
    #[arg(help = "The ID of the bind")]
    pub id: i32,
    #[arg(help = "The template to change to", rest)]
    pub template: String,
}

pub async fn cb_modify_template(ctx: CommandContext, args: ModifyTemplate) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let id_to_modify = args.id;
    let template = args.template;

    if template.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Custombind Modification Failed")
            .description("You have entered a blank template")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    ctx.bot
        .database
        .execute(
            "UPDATE binds SET template = $1 WHERE bind_id = $2",
            &[&template, &bind.bind_id],
        )
        .await?;

    let name = format!("Id: {id_to_modify}");
    let desc = format!("`Template`: {0} -> {template}", bind.template);

    cb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct AddRoles {
    #[arg(help = "The ID of the bind")]
    pub id: i32,
    #[arg(help = "The roles to add", rest)]
    pub roles: String,
}

pub async fn cb_add_roles(ctx: CommandContext, args: AddRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let id_to_modify = args.id;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
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
    let name = format!("Id: {id_to_modify}");
    let desc = format!("`Added Roles`: {modification}");

    cb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct RemoveRoles {
    #[arg(help = "The ID of the bind")]
    pub id: i32,
    #[arg(help = "The roles to remove", rest)]
    pub roles: String,
}

pub async fn cb_remove_roles(ctx: CommandContext, args: RemoveRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let id_to_modify = args.id;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
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
    let name = format!("Id: {id_to_modify}");
    let desc = format!("`Removed Roles`: {modification}");

    cb_reply_log(ctx, name, desc).await
}

async fn cb_reply_log(ctx: CommandContext, name: String, desc: String) -> CommandResult {
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
        .description("Custom Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(())
}
