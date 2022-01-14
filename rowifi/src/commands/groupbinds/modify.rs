use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Groupbind},
    id::RoleId,
};

#[derive(FromArgs)]
pub struct ModifyPriority {
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The priority to set")]
    pub priority: i32,
}

pub async fn gb_modify_priority(ctx: CommandContext, args: ModifyPriority) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    let priority = args.priority;

    let groupbinds = ctx
        .bot
        .database
        .query::<Groupbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id",
            &[&(guild_id), &BindType::Group],
        )
        .await?;

    let bind = match groupbinds.iter().find(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
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

    let name = format!("Id: {group_id}");
    let desc = format!("`Priority`: {0} -> {priority}", bind.priority);

    gb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct ModifyTemplate {
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The template to set", rest)]
    pub template: String,
}

pub async fn gb_modify_template(ctx: CommandContext, args: ModifyTemplate) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    let template = args.template;

    if template.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Groupbind Modification Failed")
            .description("You have entered a blank template")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let groupbinds = ctx
        .bot
        .database
        .query::<Groupbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id",
            &[&(guild_id), &BindType::Group],
        )
        .await?;

    let bind = match groupbinds.iter().find(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
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

    let name = format!("Id: {}", group_id);
    let desc = format!("`Template`: {0} -> {template}", bind.template);

    gb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct AddRoles {
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The roles to add", rest)]
    pub roles: String,
}

pub async fn gb_add_roles(ctx: CommandContext, args: AddRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let groupbinds = ctx
        .bot
        .database
        .query::<Groupbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id",
            &[&(guild_id), &BindType::Group],
        )
        .await?;

    let bind = match groupbinds.iter().find(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
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
    let name = format!("Id: {}", group_id);
    let desc = format!("`Added Roles`: {modification}");

    gb_reply_log(ctx, name, desc).await
}

#[derive(FromArgs)]
pub struct RemoveRoles {
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The roles to remove", rest)]
    pub roles: String,
}

pub async fn gb_remove_roles(ctx: CommandContext, args: AddRoles) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let group_id = args.group_id;
    
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let groupbinds = ctx
        .bot
        .database
        .query::<Groupbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id",
            &[&(guild_id), &BindType::Group],
        )
        .await?;

    let bind = match groupbinds.iter().find(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
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
    let name = format!("Id: {}", group_id);
    let desc = format!("`Removed Roles`: {modification}");

    gb_reply_log(ctx, name, desc).await
}

async fn gb_reply_log(ctx: CommandContext, name: String, desc: String) -> CommandResult {
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
        .description("Group Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(())
}
