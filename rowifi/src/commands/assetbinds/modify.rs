use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{Assetbind, BindType},
    id::RoleId,
};

#[derive(FromArgs)]
pub struct ModifyPriority {
    #[arg(help = "The id of the asset to modify")]
    pub asset_id: i64,
    #[arg(help = "The priority to set")]
    pub priority: i32,
}

pub async fn ab_modify_priority(ctx: CommandContext, args: ModifyPriority) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let asset_id = args.asset_id;
    let priority = args.priority;

    let assetbinds = ctx
        .bot
        .database
        .query::<Assetbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY asset_id",
            &[&(guild_id), &BindType::Asset],
        )
        .await?;

    let bind = match assetbinds.iter().find(|a| a.asset_id == asset_id) {
        Some(a) => a,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Modification Failed")
                .description(format!("A bind with Asset Id {} does not exist", asset_id))
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

    let name = format!("Id: {}", asset_id);
    let desc = format!("`Priority`: {0} -> {priority}", bind.priority);

    ab_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct ModifyTemplate {
    #[arg(help = "The id of the asset to modify")]
    pub asset_id: i64,
    #[arg(help = "The template to set", rest)]
    pub template: String,
}

pub async fn ab_modify_template(ctx: CommandContext, args: ModifyTemplate) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let asset_id = args.asset_id;
    let template = args.template;

    let assetbinds = ctx
        .bot
        .database
        .query::<Assetbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY asset_id",
            &[&(guild_id), &BindType::Asset],
        )
        .await?;

    let bind = match assetbinds.iter().find(|a| a.asset_id == asset_id) {
        Some(a) => a,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Modification Failed")
                .description(format!("A bind with Asset Id {} does not exist", asset_id))
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

    let name = format!("Id: {}", asset_id);
    let desc = format!("`Template`: {0} -> {template}", bind.template);

    ab_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct AddRoles {
    #[arg(help = "The id of the asset to modify")]
    pub asset_id: i64,
    #[arg(help = "The roles to add", rest)]
    pub roles: String,
}

pub async fn ab_add_roles(ctx: CommandContext, args: AddRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let asset_id = args.asset_id;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let assetbinds = ctx
        .bot
        .database
        .query::<Assetbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY asset_id",
            &[&(guild_id), &BindType::Asset],
        )
        .await?;

    let bind = match assetbinds.iter().find(|a| a.asset_id == asset_id) {
        Some(a) => a,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Modification Failed")
                .description(format!("A bind with Asset Id {} does not exist", asset_id))
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
    let name = format!("Id: {}", asset_id);
    let desc = format!("`Added Roles`: {modification}");

    ab_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct RemoveRoles {
    #[arg(help = "The id of the asset to modify")]
    pub asset_id: i64,
    #[arg(help = "The roles to remove", rest)]
    pub roles: String,
}

pub async fn ab_remove_roles(ctx: CommandContext, args: RemoveRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let asset_id = args.asset_id;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let assetbinds = ctx
        .bot
        .database
        .query::<Assetbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY asset_id",
            &[&(guild_id), &BindType::Asset],
        )
        .await?;

    let bind = match assetbinds.iter().find(|a| a.asset_id == asset_id) {
        Some(a) => a,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Modification Failed")
                .description(format!("A bind with Asset Id {} does not exist", asset_id))
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
    let name = format!("Id: {}", asset_id);
    let desc = format!("`Removed Roles`: {modification}");

    ab_reply_log(ctx, name, desc).await
}

async fn ab_reply_log(ctx: CommandContext, name: String, desc: String) -> CommandResult {
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
        .description("Asset Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(())
}
