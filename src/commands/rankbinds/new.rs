use crate::framework::prelude::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use twilight_model::id::RoleId;
use twilight_embed_builder::EmbedFieldBuilder;

use crate::models::{bind::RankBind, guild::RoGuild};

pub static RANKBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to add a new rankbind"),
    usage: Some("rankbinds new <Group Id> <Rank Id (1-255)> <Prefix> <Priority> [Roles..]`\n
                `Rank Id`: Either a single rank id between 1-255 or a range of rank ids separated by a `-`. Ex. 25-55\n
                `Prefix`: The keyword that is used before the nickname. Can be set to `N/A`\n
                `Priorty`: The number that tells the bot which rankbind to choose for the nickname\n
                `Roles`: The discord roles to add to the bind. To tell the bot to create roles, put `auto"),
    examples: &["rankbinds new 3108077 255 [CJCS] 1 @CJCS", "rb new 5581309 1-255 N/A 1 @RoWifi",
                "rb new 5581309 1-255 auto 1 @RoWifi", "rankbinds new 3108077 1-255 auto 1 auto"],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static RANKBINDS_NEW_COMMAND: Command = Command {
    fun: rankbinds_new,
    options: &RANKBINDS_NEW_OPTIONS
};

lazy_static! {
    static ref PREFIX_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}

#[command]
pub async fn rankbinds_new(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let mut create_type = match args.next() {
        None => return Ok(()),
        Some(s) => {
            if let Ok(r) = s.parse::<i64>() {
                CreateType::Single(r)
            } else {
                match extract_ids(s) {
                    Some((r1, r2)) => CreateType::Multiple(r1, r2),
                    None => return Ok(())
                }
            }
        },
    };

    let prefix = match args.next() {
        Some(p) => p.to_owned(),
        None => return Ok(())
    };

    let priority = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(p)) => p,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let mut roles: Vec<i64> = Vec::new();
    for r in args {
        if r.eq_ignore_ascii_case("auto") {
            if let CreateType::Single(r1) = create_type {create_type = CreateType::SingleWithAuto(r1);}
            else if let CreateType::Multiple(r1, r2) = create_type {create_type = CreateType::MultipleWithAuto(r1, r2);}
            break;
        }
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    match create_type {
        CreateType::Single(rank_id) => single_rank(ctx, msg, guild, group_id, rank_id, prefix, priority, roles).await?,
        CreateType::SingleWithAuto(rank_id) => single_rank_with_auto(ctx, msg, group_id, rank_id, prefix, priority).await?,
        CreateType::Multiple(min_rank, max_rank) => multiple_rank(ctx, msg, guild, group_id, min_rank, max_rank, &prefix, priority, roles).await?,
        CreateType::MultipleWithAuto(min_rank, max_rank) => multiple_rank_with_auto(ctx, msg, guild, group_id, min_rank, max_rank, &prefix, priority).await?
    };

    Ok(())
}

#[derive(Eq, PartialEq)]
#[repr(i8)]
enum CreateType {
    Single(i64), SingleWithAuto(i64), Multiple(i64, i64), MultipleWithAuto(i64, i64)
}

#[allow(clippy::too_many_arguments)]
async fn single_rank(ctx: &Context, msg: &Message, guild: RoGuild, group_id: i64, rank_id: i64, mut prefix: String, priority: i64, roles: Vec<i64>) -> Result<(), RoError> {
    if guild.rankbinds.iter().any(|r| r.group_id == group_id && r.rank_id == rank_id) {
        return Ok(())
    }
    
    let roblox_rank = match ctx.roblox.get_group_rank(group_id, rank_id).await?{
        Some(r) => r,
        None => return Ok(())
    };
    
    if prefix.eq("auto") {
        prefix = match PREFIX_REGEX.captures(roblox_rank["name"].as_str().unwrap()) {
            Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
            None => "N/A".into()
        };
    }

    let bind = RankBind {
        group_id,
        rank_id,
        rbx_rank_id: roblox_rank["id"].as_i64().unwrap(),
        prefix: prefix.clone(),
        priority,
        discord_roles: roles
    };
    println!("{:?}", bind);
    add_rankbind(ctx, msg, bind).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn single_rank_with_auto(ctx: &Context, msg: &Message, group_id: i64, rank_id: i64, mut prefix: String, priority: i64) -> Result<(), RoError> {
    let roblox_rank = match ctx.roblox.get_group_rank(group_id, rank_id).await?{
        Some(r) => r,
        None => return Ok(())
    };
    
    if prefix.eq("auto") {
        prefix = match PREFIX_REGEX.captures(roblox_rank["name"].as_str().unwrap()) {
            Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
            None => "N/A".into()
        };
    }

    let server_roles = ctx.cache.guild_roles(msg.guild_id.unwrap());
    let role = match server_roles.iter().find(|r| r.name.eq_ignore_ascii_case(roblox_rank["name"].as_str().unwrap())) {
        Some(r) => r.id.0 as i64,
        None => {
            let new_role = ctx.http.create_role(msg.guild_id.unwrap()).name(roblox_rank["name"].as_str().unwrap()).await?;
            new_role.id.0 as i64
        }
    };
    let discord_roles = vec![role];
    let bind = RankBind {
        group_id,
        rank_id,
        rbx_rank_id: roblox_rank["id"].as_i64().unwrap(),
        prefix: prefix.clone(),
        priority,
        discord_roles
    };
    add_rankbind(ctx, msg, bind).await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn multiple_rank(ctx: &Context, msg: &Message, mut guild: RoGuild, group_id: i64, min_rank: i64, max_rank: i64, prefix: &str, priority: i64, roles: Vec<i64>) -> Result<(), RoError> {
    let roblox_ranks = ctx.roblox.get_group_ranks(group_id, min_rank, max_rank).await?;
    if roblox_ranks.is_empty() {
        return Ok(())
    }

    let mut added = 0;
    let mut modified = 0;
    for roblox_rank in roblox_ranks {
        let mut prefix_to_set = prefix.to_string();
        if prefix.eq("auto") {
            prefix_to_set = match PREFIX_REGEX.captures(roblox_rank["name"].as_str().unwrap()) {
                Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
                None => "N/A".into()
            };
        }
        let rank_id = roblox_rank["rank"].as_i64().unwrap();
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: roblox_rank["id"].as_i64().unwrap(),
            prefix: prefix_to_set.clone(),
            priority,
            discord_roles: roles.clone()
        };

        match guild.rankbinds.iter().find_position(|r| r.group_id == group_id && r.rank_id == rank_id) {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind;
                modified += 1;
            },
            None => {
                guild.rankbinds.push(bind);
                added += 1;
            }
        }
    }

    ctx.database.add_guild(guild, true).await?;
    let embed = EmbedBuilder::new().default_data().title("Binds Addition Sucessful").unwrap()
        .color(Color::Red as u32).unwrap()
        .description(format!("Added {} rankbinds and modified {} rankbinds", added, modified)).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn multiple_rank_with_auto(ctx: &Context, msg: &Message, mut guild: RoGuild, group_id: i64, min_rank: i64, max_rank: i64, prefix: &str, priority: i64) ->Result<(), RoError> {
    let roblox_ranks = ctx.roblox.get_group_ranks(group_id, min_rank, max_rank).await?;
    if roblox_ranks.is_empty() {
        return Ok(())
    }

    let mut added = 0;
    let mut modified = 0;
    for roblox_rank in roblox_ranks {
        let mut prefix_to_set = prefix.to_string();
        if prefix.eq("auto") {
            prefix_to_set = match PREFIX_REGEX.captures(roblox_rank["name"].as_str().unwrap()) {
                Some(m) => format!("[{}]", m.get(1).unwrap().as_str()),
                None => "N/A".into()
            };
        }
        let rank_id = roblox_rank["rank"].as_i64().unwrap();

        let server_roles = ctx.cache.guild_roles(msg.guild_id.unwrap());
        let role = match server_roles.iter().find(|r| r.name.eq_ignore_ascii_case(roblox_rank["name"].as_str().unwrap())) {
            Some(r) => r.id.0 as i64,
            None => {
                let new_role = ctx.http.create_role(msg.guild_id.unwrap()).name(roblox_rank["name"].as_str().unwrap()).await?;
                new_role.id.0 as i64
            }
        };
        let discord_roles = vec![role];
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: roblox_rank["id"].as_i64().unwrap(),
            prefix: prefix_to_set.clone(),
            priority,
            discord_roles
        };

        match guild.rankbinds.iter().find_position(|r| r.group_id == group_id && r.rank_id == rank_id) {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind;
                modified += 1;
            },
            None => {
                guild.rankbinds.push(bind);
                added += 1;
            }
        }
    }

    ctx.database.add_guild(guild, true).await?;
    let embed = EmbedBuilder::new().default_data().title("Binds Addition Sucessful").unwrap()
        .color(Color::Red as u32).unwrap()
        .description(format!("Added {} rankbinds and modified {} rankbinds", added, modified)).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

async fn add_rankbind(ctx: &Context, msg: &Message, bind: RankBind) -> Result<(), RoError> {
    let filter = bson::doc! {"_id": msg.guild_id.unwrap().0 };
    let bind_bson = bson::to_bson(&bind)?;
    let update = bson::doc! {"$push": {"RankBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Rank: {}", bind.rank_id);
    let roles_str = bind.discord_roles.iter().map(|r| format!("<@&{}> ", r)).collect::<String>();
    let desc = format!("Prefix: {}\nPriority: {}\nDiscord Roles: {}", bind.prefix, bind.priority, roles_str);
    let embed = EmbedBuilder::new().default_data().title("Bind Addition Sucessful").unwrap()
        .color(Color::Red as u32).unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;
    Ok(())
}

fn extract_ids(rank_str: &str) -> Option<(i64, i64)> {
    let splits = rank_str.split('-').collect_vec();
    if splits.len() == 2 {
        if let Ok(r1) = splits[0].parse::<i64>() {
            if let Ok(r2) = splits[1].parse::<i64>() {
                return Some((r1, r2))
            }
        }
    }
    None
}