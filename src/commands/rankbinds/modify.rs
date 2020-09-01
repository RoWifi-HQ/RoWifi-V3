use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static RANKBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["modify", "m"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static RANKBINDS_MODIFY_COMMAND: Command = Command {
    fun: rankbinds_modify,
    options: &RANKBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn rankbinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild = match ctx.database.get_guild(msg.guild_id.unwrap().0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
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

    if !guild.rankbinds.iter().any(|r| r.group_id == group_id && r.rank_id == rank_id) {
        return Ok(())
    }

    if field.eq_ignore_ascii_case("prefix") {
        modify_prefix(ctx, &guild, group_id, rank_id, args.next()).await?;
    } else if field.eq_ignore_ascii_case("priority") {
        modify_priority(ctx, &guild, group_id, rank_id, args.next()).await?;
    } else if field.eq_ignore_ascii_case("roles-add") {
        add_roles(ctx, &guild, group_id, rank_id, args).await?;
    } else if field.eq_ignore_ascii_case("roles-remove") {
        remove_roles(ctx, &guild, group_id, rank_id, args).await?;
    } 

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The bind was successfully modified").unwrap()
        .build().unwrap();

    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}

async fn modify_prefix(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, prefix: Option<&str>) -> Result<(), RoError> {
    let prefix = match prefix {
        Some(s) => s,
        None => return Ok(())
    };
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$set": {"RankBinds.$.Prefix": prefix}};
    ctx.database.modify_guild(filter, update).await
}

async fn modify_priority(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, priority: Option<&str>) -> Result<(), RoError> {
    let priority = match priority.map(|p| p.parse::<i64>()) {
        Some(Ok(p)) => p,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$set": {"RankBinds.$.Priority": priority}};
    ctx.database.modify_guild(filter, update).await
}

async fn add_roles(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$push": {"RankBinds.$.DiscordRoles": {"$each": role_ids}}};
    ctx.database.modify_guild(filter, update).await
}

async fn remove_roles(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$pullAll": {"RankBinds.$.DiscordRoles": role_ids}};
    ctx.database.modify_guild(filter, update).await
}