use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static RANKBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify", "m"],
    desc: Some("Command to modify a rankbind"),
    usage: Some("rankbinds modify <Field> <Bind Id> [Params...]`\n`Field`: `priority`, `prefix`, `roles-add`, `roles-remove"),
    examples: &["rankbinds modify priority 3108077 255 2", "rb modify prefix 5581309 35 [CUS]", "rankbinds m roles-add 5581309 255 @Role1"],
    required_permissions: Permissions::empty(),
    min_args: 3,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static RANKBINDS_MODIFY_COMMAND: Command = Command {
    fun: rankbinds_modify,
    options: &RANKBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn rankbinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
    };

    let group_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Group ID".into(), "Number".into()).into())
        },
        None => return Ok(())
    };

    let rank_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Rank ID".into(), "Number".into()).into())
        },
        None => return Ok(())
    };

    let bind = match guild.rankbinds.iter().find(|r| r.group_id == group_id && r.rank_id == rank_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                .title("Rank Bind Modification Failed").unwrap()
                .description(format!("There was no bind found with Group Id {} and Rank Id {}", group_id, rank_id)).unwrap()
                .build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
            return Ok(())
        }
    };

    let name = format!("Group Id: {}", bind.group_id);
    let desc = if field.eq_ignore_ascii_case("prefix") {
        let new_prefix = modify_prefix(ctx, &guild, group_id, rank_id, args.next()).await?;
        format!("`Prefix`: {} -> {}", bind.prefix, new_prefix)
    } else if field.eq_ignore_ascii_case("priority") {
        let new_priority = modify_priority(ctx, &guild, group_id, rank_id, args.next()).await?;
        format!("`Priority`: {} -> {}", bind.priority, new_priority)
    } else if field.eq_ignore_ascii_case("roles-add") {
        let role_ids = add_roles(ctx, &guild, group_id, rank_id, args).await?;
        let modification = role_ids.iter().map(|r| format!("<@&{}> ", r)).collect::<String>();
        format!("Added Roles: {}", modification)
    } else if field.eq_ignore_ascii_case("roles-remove") {
        let role_ids = remove_roles(ctx, &guild, group_id, rank_id, args).await?;
        let modification = role_ids.iter().map(|r| format!("<@&{}> ", r)).collect::<String>();
        format!("Removed Roles: {}", modification)
    } else {
        return Err(CommandError::ParseArgument(field, "Field".into(), "`prefix`, `priority`, `roles-add`, `roles-remove`".into()).into())
    };
    let desc = format!("Rank Id: {}\n{}", bind.rank_id, desc);

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The bind was successfully modified").unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description("Rank Bind Modification").unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

async fn modify_prefix(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, prefix: Option<&str>) -> Result<String, RoError> {
    let prefix = prefix.unwrap();
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$set": {"RankBinds.$.Prefix": prefix}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(prefix.to_string())
}

async fn modify_priority(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, priority: Option<&str>) -> Result<i64, RoError> {
    let priority = match priority.unwrap().parse::<i64>() {
        Ok(p) => p,
        Err(_) => return Err(CommandError::ParseArgument(priority.unwrap().into(), "Priority".into(), "Number".into()).into())
    };
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$set": {"RankBinds.$.Priority": priority}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn add_roles(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, args: Arguments<'_>) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$push": {"RankBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, args: Arguments<'_>) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$pullAll": {"RankBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}