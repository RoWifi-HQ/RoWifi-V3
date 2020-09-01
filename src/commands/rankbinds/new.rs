use crate::framework::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use twilight_model::id::RoleId;
use twilight_embed_builder::EmbedFieldBuilder;

use crate::models::bind::RankBind;

pub static RANKBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["new"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static RANKBINDS_NEW_COMMAND: Command = Command {
    fun: rankbinds_new,
    options: &RANKBINDS_NEW_OPTIONS
};

#[command]
pub async fn rankbinds_new(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild = match ctx.database.get_guild(msg.guild_id.unwrap().0).await? {
        Some(g) => g,
        None => {
            return Ok(())
        }
    };

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let rank_id = match args.next().map(|r| r.parse::<i64>()) {
        Some(Ok(s)) => s,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let mut prefix = match args.next() {
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
    while let Some(r) = args.next() {
        if let Ok(role_id) = r.parse::<i64>() {
            if server_roles.contains(&RoleId(role_id as u64)) {
                roles.push(role_id);
            }
        }
    }

    let roblox_rank = match ctx.roblox.get_group_rank(group_id, rank_id).await?{
        Some(r) => r,
        None => return Ok(())
    };
    
    if prefix.eq("auto") {
        prefix = match PREFIX_REGEX.captures(&prefix).unwrap().get(1) {
            Some(m) => m.as_str().to_owned(),
            None => "N/A".into()
        };
    }

    let bind = RankBind {
        group_id,
        rank_id,
        rbx_rank_id: roblox_rank["id"].as_i64().unwrap(),
        prefix,
        priority,
        discord_roles: roles
    };
    let filter = bson::doc! {"_id": guild.id };
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

lazy_static! {
    static ref PREFIX_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}