use itertools::Itertools;
use lazy_static::lazy_static;
use mongodb::bson::{doc, to_bson};
use regex::Regex;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::RankBind,
    guild::RoGuild,
    roblox::{group::PartialRank, id::GroupId},
};
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_model::id::RoleId;

#[derive(Debug, FromArgs)]
pub struct NewRankbind {
    #[arg(help = "The Group ID of your Roblox Group")]
    pub group_id: i64,
    #[arg(
        help = "Either a single rank id between 1-255 or a range of rank ids separated by a `-`. Ex. 25-55"
    )]
    pub rank_id: CreateType,
    #[arg(help = "The keyword that is used before the nickname. Can be set to `N/A` or `default`")]
    pub prefix: String,
    #[arg(help = "The number that tells the bot which rankbind to choose for the nickname")]
    pub priority: Option<i64>,
    #[arg(
        help = "The discord roles to add to the bind. To tell the bot to create roles, put `auto` ",
        rest
    )]
    pub discord_roles: Option<String>,
}

lazy_static! {
    static ref PREFIX_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}

pub async fn rankbinds_new(ctx: CommandContext, args: NewRankbind) -> Result<(), RoError> {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let group_id = args.group_id;
    let mut create_type = args.rank_id;
    let prefix = args.prefix;
    let priority = args.priority.unwrap_or_default();
    let discord_roles = args.discord_roles.unwrap_or_default();

    let server_roles = ctx.bot.cache.roles(ctx.guild_id.unwrap());
    let mut roles: Vec<i64> = Vec::new();
    for r in discord_roles.split_ascii_whitespace() {
        if r.eq_ignore_ascii_case("auto") {
            if let CreateType::Single(r1) = create_type {
                create_type = CreateType::SingleWithAuto(r1);
            } else if let CreateType::Multiple(r1, r2) = create_type {
                create_type = CreateType::MultipleWithAuto(r1, r2);
            }
            break;
        }
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    match create_type {
        CreateType::Single(rank_id) => {
            single_rank(ctx, guild, group_id, rank_id, prefix, priority, roles).await?
        }
        CreateType::SingleWithAuto(rank_id) => {
            single_rank_with_auto(ctx, guild, group_id, rank_id, prefix, priority).await?
        }
        CreateType::Multiple(min_rank, max_rank) => {
            multiple_rank(
                ctx, guild, group_id, min_rank, max_rank, &prefix, priority, roles,
            )
            .await?
        }
        CreateType::MultipleWithAuto(min_rank, max_rank) => {
            multiple_rank_with_auto(ctx, guild, group_id, min_rank, max_rank, &prefix, priority)
                .await?
        }
    };

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum CreateType {
    Single(i64),
    SingleWithAuto(i64),
    Multiple(i64, i64),
    MultipleWithAuto(i64, i64),
}

impl FromArg for CreateType {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        if let Ok(r) = arg.parse::<i64>() {
            Ok(CreateType::Single(r))
        } else if let Some((min_rank, max_rank)) = extract_ids(arg) {
            Ok(CreateType::Multiple(min_rank, max_rank))
        } else {
            Err(ParseError("a number or a range (1-255)"))
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::Integer { value, .. } => value.to_string(),
            CommandDataOption::String { value, .. } => value.to_string(),
            _ => unreachable!("NewRankbind unreached"),
        };

        CreateType::from_arg(&arg)
    }
}

#[allow(clippy::too_many_arguments)]
async fn single_rank(
    ctx: CommandContext,
    guild: RoGuild,
    group_id: i64,
    rank_id: i64,
    mut prefix: String,
    priority: i64,
    roles: Vec<i64>,
) -> Result<(), RoError> {
    if guild
        .rankbinds
        .iter()
        .any(|r| r.group_id == group_id && r.rank_id == rank_id)
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Rankbind Addition Failed")
            .unwrap()
            .description(format!(
                "A rankbind with group id {} and rank id {} already exists",
                group_id, rank_id
            ))
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let roblox_rank = match get_group_rank(&ctx, GroupId(group_id as u64), rank_id).await? {
        Some(r) => r,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Rankbind Addition Failed")
                .unwrap()
                .description(format!(
                    "The Rank {} does not exist in Group {}",
                    rank_id, group_id
                ))
                .unwrap()
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    if prefix.eq("auto") {
        prefix = match PREFIX_REGEX.captures(&roblox_rank.name) {
            Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
            None => "N/A".into(),
        };
    }

    let bind = RankBind {
        group_id,
        rank_id,
        rbx_rank_id: roblox_rank.id.0 as i64,
        prefix: Some(prefix.clone()),
        priority,
        discord_roles: roles,
        template: None,
    };
    add_rankbind(&ctx, &bind).await?;
    log_rankbind(&ctx, bind).await;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn single_rank_with_auto(
    ctx: CommandContext,
    guild: RoGuild,
    group_id: i64,
    rank_id: i64,
    mut prefix: String,
    priority: i64,
) -> Result<(), RoError> {
    if guild
        .rankbinds
        .iter()
        .any(|r| r.group_id == group_id && r.rank_id == rank_id)
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Rankbind Addition Failed")
            .unwrap()
            .description(format!(
                "A rankbind with group id {} and rank id {} already exists",
                group_id, rank_id
            ))
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let roblox_rank = match get_group_rank(&ctx, GroupId(group_id as u64), rank_id).await? {
        Some(r) => r,
        None => return Ok(()),
    };

    if prefix.eq("auto") {
        prefix = match PREFIX_REGEX.captures(&roblox_rank.name) {
            Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
            None => "N/A".into(),
        };
    }

    let server_roles = ctx.bot.cache.guild_roles(ctx.guild_id.unwrap());
    let role = match server_roles
        .iter()
        .find(|r| r.name.eq_ignore_ascii_case(&roblox_rank.name))
    {
        Some(r) => r.id.0 as i64,
        None => {
            let new_role = ctx
                .bot
                .http
                .create_role(ctx.guild_id.unwrap())
                .name(roblox_rank.name)
                .await?;
            new_role.id.0 as i64
        }
    };
    let discord_roles = vec![role];
    let bind = RankBind {
        group_id,
        rank_id,
        rbx_rank_id: roblox_rank.id.0 as i64,
        prefix: Some(prefix.clone()),
        priority,
        discord_roles,
        template: None,
    };
    add_rankbind(&ctx, &bind).await?;
    log_rankbind(&ctx, bind).await;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn multiple_rank(
    ctx: CommandContext,
    mut guild: RoGuild,
    group_id: i64,
    min_rank: i64,
    max_rank: i64,
    prefix: &str,
    priority: i64,
    roles: Vec<i64>,
) -> Result<(), RoError> {
    let roblox_ranks = get_group_ranks(&ctx, GroupId(group_id as u64), min_rank, max_rank).await?;
    if roblox_ranks.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Rankbind Addition Failed")
            .unwrap()
            .description("There were no ranks found in the given range")
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();
    for roblox_rank in roblox_ranks {
        let mut prefix_to_set = prefix.to_string();
        if prefix.eq("auto") {
            prefix_to_set = match PREFIX_REGEX.captures(&roblox_rank.name) {
                Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
                None => "N/A".into(),
            };
        }
        let rank_id = roblox_rank.rank as i64;
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: roblox_rank.id.0 as i64,
            prefix: Some(prefix_to_set.clone()),
            priority,
            discord_roles: roles.clone(),
            template: None,
        };

        match guild
            .rankbinds
            .iter()
            .find_position(|r| r.group_id == group_id && r.rank_id == rank_id)
        {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind.clone();
                modified.push(bind)
            }
            None => {
                guild.rankbinds.push(bind.clone());
                added.push(bind);
            }
        }
    }

    ctx.bot.database.add_guild(guild, true).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Binds Addition Sucessful")
        .unwrap()
        .color(Color::Red as u32)
        .unwrap()
        .description(format!(
            "Added {} rankbinds and modified {} rankbinds",
            added.len(),
            modified.len()
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    for rb in added {
        log_rankbind(&ctx, rb).await;
    }
    for rb in modified {
        log_rankbind(&ctx, rb).await;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn multiple_rank_with_auto(
    ctx: CommandContext,
    mut guild: RoGuild,
    group_id: i64,
    min_rank: i64,
    max_rank: i64,
    prefix: &str,
    priority: i64,
) -> Result<(), RoError> {
    let roblox_ranks = get_group_ranks(&ctx, GroupId(group_id as u64), min_rank, max_rank).await?;
    if roblox_ranks.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Rankbind Addition Failed")
            .unwrap()
            .description("There were no ranks found in the given range")
            .unwrap()
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let mut added = Vec::new();
    let mut modified = Vec::new();
    for roblox_rank in roblox_ranks {
        let mut prefix_to_set = prefix.to_string();
        if prefix.eq("auto") {
            prefix_to_set = match PREFIX_REGEX.captures(&roblox_rank.name) {
                Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
                None => "N/A".into(),
            };
        }
        let rank_id = roblox_rank.rank as i64;

        let server_roles = ctx.bot.cache.guild_roles(ctx.guild_id.unwrap());
        let role = match server_roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(&roblox_rank.name))
        {
            Some(r) => r.id.0 as i64,
            None => {
                let new_role = ctx
                    .bot
                    .http
                    .create_role(ctx.guild_id.unwrap())
                    .name(roblox_rank.name)
                    .await?;
                new_role.id.0 as i64
            }
        };
        let discord_roles = vec![role];
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: roblox_rank.id.0 as i64,
            prefix: Some(prefix_to_set.clone()),
            priority,
            discord_roles,
            template: None,
        };

        match guild
            .rankbinds
            .iter()
            .find_position(|r| r.group_id == group_id && r.rank_id == rank_id)
        {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind.clone();
                modified.push(bind);
            }
            None => {
                guild.rankbinds.push(bind.clone());
                added.push(bind)
            }
        }
    }

    ctx.bot.database.add_guild(guild, true).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Binds Addition Sucessful")
        .unwrap()
        .color(Color::Red as u32)
        .unwrap()
        .description(format!(
            "Added {} rankbinds and modified {} rankbinds",
            added.len(),
            modified.len()
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    for rb in added {
        log_rankbind(&ctx, rb).await;
    }
    for rb in modified {
        log_rankbind(&ctx, rb).await;
    }
    Ok(())
}

async fn add_rankbind(ctx: &CommandContext, bind: &RankBind) -> Result<(), RoError> {
    let filter = doc! {"_id": ctx.guild_id.unwrap().0 };
    let bind_bson = to_bson(&bind)?;
    let update = doc! {"$push": {"RankBinds": bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Rank: {}", bind.rank_id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Prefix: {}\nPriority: {}\nDiscord Roles: {}",
        bind.prefix.as_ref().map_or("", |s| s.as_str()),
        bind.priority,
        roles_str
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Sucessful")
        .unwrap()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    Ok(())
}

async fn log_rankbind(ctx: &CommandContext, bind: RankBind) {
    let name = format!("Group Id: {}", bind.group_id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Rank Id: {}\nPrefix: {}\nPriority: {}\nDiscord Roles: {}",
        bind.rank_id,
        bind.prefix.unwrap_or_default(),
        bind.priority,
        roles_str
    );
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Rank Bind Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(ctx.guild_id.unwrap(), log_embed).await;
}

fn extract_ids(rank_str: &str) -> Option<(i64, i64)> {
    let splits = rank_str.split('-').collect_vec();
    if splits.len() == 2 {
        if let Ok(r1) = splits[0].parse::<i64>() {
            if let Ok(r2) = splits[1].parse::<i64>() {
                return Some((r1, r2));
            }
        }
    }
    None
}

async fn get_group_rank(
    ctx: &CommandContext,
    group_id: GroupId,
    rank_id: i64,
) -> Result<Option<PartialRank>, RoError> {
    let group = ctx.bot.roblox.get_group_ranks(group_id).await?;
    match group {
        None => Ok(None),
        Some(g) => Ok(g.roles.into_iter().find(|r| r.rank as i64 == rank_id)),
    }
}

async fn get_group_ranks(
    ctx: &CommandContext,
    group_id: GroupId,
    min_rank: i64,
    max_rank: i64,
) -> Result<Vec<PartialRank>, RoError> {
    let group = ctx.bot.roblox.get_group_ranks(group_id).await?;
    match group {
        None => Ok(Vec::new()),
        Some(g) => Ok(g
            .roles
            .into_iter()
            .filter(|r| r.rank as i64 >= min_rank && r.rank as i64 <= max_rank)
            .collect()),
    }
}
