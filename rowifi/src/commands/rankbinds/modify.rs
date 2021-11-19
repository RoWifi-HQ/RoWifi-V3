use itertools::Itertools;
use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::{guild::RoGuild, roblox::id::GroupId};

use super::new::PREFIX_REGEX;

#[derive(FromArgs)]
pub struct ModifyRankbind {
    #[arg(
        help = "The field to modify. Must be one of `priority` `roles-add` `roles-remove` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The Group ID of the rankbind to modify")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the rankbind to modify")]
    pub rank_id: i64,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    Priority,
    RolesAdd,
    RolesRemove,
    Template,
}

pub async fn rankbinds_modify(ctx: CommandContext, args: ModifyRankbind) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let group_id = args.group_id;
    let rank_id = args.rank_id;

    let bind_index = match guild
        .rankbinds
        .iter()
        .position(|r| r.group_id == group_id && r.rank_id == rank_id)
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
    let bind = &guild.rankbinds[bind_index];

    let name = format!("Group Id: {}", bind.group_id);
    let desc = match args.option {
        ModifyOption::Priority => {
            let priority = match args.change.parse::<i64>() {
                Ok(p) => p,
                Err(_) => {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::Red as u32)
                        .title("Rank Bind Modification Failed")
                        .description(format!("Priority was not found to be a number",))
                        .build()
                        .unwrap();
                    ctx.respond().embeds(&[embed])?.exec().await?;
                    return Ok(());
                }
            };
            let new_priority = modify_priority(&ctx, &guild, bind_index, priority).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, &guild, bind_index, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Added Roles: {}", modification)
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, &guild, bind_index, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Removed Roles: {}", modification)
        }
        ModifyOption::Template => {
            if args.change.is_empty() {
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
            let template =
                modify_template(&ctx, group_id, rank_id, &guild, bind_index, &args.change).await?;
            format!("`New Template`: {}", template)
        }
    };
    let desc = format!("Rank Id: {}\n{}", bind.rank_id, desc);

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

async fn modify_priority(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    priority: i64,
) -> Result<i64, RoError> {
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Priority", bind_index);
    let update = doc! {"$set": {index_str: priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    group_id: i64,
    rank_id: i64,
    guild: &RoGuild,
    bind_index: usize,
    template: &'t str,
) -> Result<String, RoError> {
    let roblox_group = ctx
        .bot
        .roblox
        .get_group_ranks(GroupId(group_id as u64))
        .await?;
    let roblox_rank = match &roblox_group {
        Some(g) => g.roles.iter().find(|r| i64::from(r.rank) == rank_id),
        None => None,
    };
    let template = match template {
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
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Template", bind_index);
    let update = doc! {"$set": {index_str: template.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(template)
}

async fn add_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| r.id.get() as i64));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.DiscordRoles", bind_index);
    let update = doc! {"$push": {index_str: {"$each": role_ids.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| r.id.get() as i64));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.DiscordRoles", bind_index);
    let update = doc! {"$pullAll": {index_str: role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `priority` `roles-add` `roles-remove` `template`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match &option.value {
            CommandOptionValue::String(value) => value.to_string(),
            CommandOptionValue::Integer(value) => value.to_string(),
            _ => unreachable!("ModifyArgumentRankbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
