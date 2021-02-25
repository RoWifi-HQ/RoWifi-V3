use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;
use twilight_embed_builder::EmbedFieldBuilder;

#[derive(FromArgs)]
pub struct ModifyRankbind {
    #[arg(
        help = "The field to modify. Must be one of `prefix` `priority` `roles-add` `roles-remove`"
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
    Prefix,
    Priority,
    RolesAdd,
    RolesRemove,
    Template
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
            format!("`Prefix`: {} -> {}", bind.prefix.as_ref().map_or("", |s| s.as_str()), new_prefix)
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
        },
        ModifyOption::Template => {
            let template = modify_template(&ctx, &guild, bind_index, &args.change).await?;
            format!("`New Template`: {}", template)
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

async fn modify_prefix<'p>(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    prefix: &'p str,
) -> Result<&'p str, RoError> {
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Prefix", bind_index);
    let update = doc! {"$set": {index_str: prefix}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(prefix)
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
            return Err(RoError::Command(CommandError::Miscellanous("Given priority was found not to be a number".into())))
        }
    };
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Priority", bind_index);
    let update = doc! {"$set": {index_str: priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn modify_template<'t>(ctx: &CommandContext, guild: &RoGuild, bind_index: usize, template: &'t str) -> Result<&'t str, RoError> {
    let filter = doc! {"_id": guild.id};
    let index_str = format!("RankBinds.{}.Template", bind_index);
    let update = doc! {"$set": {index_str: template}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(template)
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
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
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
            "prefix" => Ok(ModifyOption::Prefix),
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `prefix` `priority` `roles-add` `roles-remove`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("ModifyArgumentRankbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
