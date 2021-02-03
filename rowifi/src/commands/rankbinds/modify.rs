use framework_new::prelude::*;
use rowifi_models::guild::RoGuild;
use twilight_embed_builder::EmbedFieldBuilder;

// pub static RANKBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
//     perm_level: RoLevel::Admin,
//     bucket: None,
//     names: &["modify", "m"],
//     desc: Some("Command to modify a rankbind"),
//     usage: Some("rankbinds modify <Field> <Bind Id> [Params...]`\n`Field`: `priority`, `prefix`, `roles-add`, `roles-remove"),
//     examples: &["rankbinds modify priority 3108077 255 2", "rb modify prefix 5581309 35 [CUS]", "rankbinds m roles-add 5581309 255 @Role1"],
//     min_args: 3,
//     hidden: false,
//     sub_commands: &[],
//     group: None
// };

#[derive(FromArgs)]
pub struct ModifyRankbind {
    pub option: ModifyOption,
    pub group_id: i64,
    pub rank_id: i64,
    pub change: String,
}

pub enum ModifyOption {
    Prefix,
    Priority,
    RolesAdd,
    RolesRemove,
}

pub async fn rankbinds_modify(ctx: CommandContext, args: ModifyRankbind) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

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
                .unwrap()
                .title("Rank Bind Modification Failed")
                .unwrap()
                .description(format!(
                    "There was no bind found with Group Id {} and Rank Id {}",
                    group_id, rank_id
                ))
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    let bind = &guild.rankbinds[bind_index];

    let name = format!("Group Id: {}", bind.group_id);
    let desc = match args.option {
        ModifyOption::Prefix => {
            let new_prefix = modify_prefix(&ctx, &guild, bind_index, &args.change).await?;
            format!("`Prefix`: {} -> {}", bind.prefix, new_prefix)
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, &guild, bind_index, &args.change).await?;
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
    };
    let desc = format!("Rank Id: {}\n{}", bind.rank_id, desc);

    let e = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Success!")
        .unwrap()
        .description("The bind was successfully modified")
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Rank Bind Modification")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

async fn modify_prefix(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    prefix: &str,
) -> Result<String, RoError> {
    let filter = bson::doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Prefix", bind_index);
    let update = bson::doc! {"$set": {index_str: prefix}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(prefix.to_string())
}

async fn modify_priority(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    priority: &str,
) -> Result<i64, RoError> {
    let priority = match priority.parse::<i64>() {
        Ok(p) => p,
        Err(_) => {
            unimplemented!()
        }
    };
    let filter = bson::doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Priority", bind_index);
    let update = bson::doc! {"$set": {index_str: priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn add_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    roles: &str,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.DiscordRoles", bind_index);
    let update = bson::doc! {"$push": {index_str: {"$each": role_ids.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    roles: &str,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.DiscordRoles", bind_index);
    let update = bson::doc! {"$pullAll": {index_str: role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg {
            "prefix" => Ok(ModifyOption::Prefix),
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            _ => Err(ParseError(
                "one of `prefix` `priority` `roles-add` `roles-remove`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("ModifyArgument unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
