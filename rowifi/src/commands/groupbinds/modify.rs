use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

#[derive(FromArgs)]
pub struct GroupbindsModifyArguments {
    #[arg(
        help = "The field to modify. Must be one of `roles-add` `roles-remove` `priority` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    RolesAdd,
    RolesRemove,
    Priority,
    Template,
}

pub async fn groupbinds_modify(
    ctx: CommandContext,
    args: GroupbindsModifyArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let field = args.option;
    let group_id = args.group_id;
    let bind_index = match guild.groupbinds.iter().position(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };
    let bind = &guild.groupbinds[bind_index];

    let name = format!("Id: {}", group_id);
    let desc = match field {
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, &guild, bind_index, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Added Roles: {}", modification);
            desc
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, &guild, bind_index, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Removed Roles: {}", modification);
            desc
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, &guild, bind_index, &args.change).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::Template => {
            if args.change.is_empty() {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .title("Group Bind Modification Failed")
                    .description("You have entered a blank template")
                    .build()
                    .unwrap();
                ctx.respond().embed(embed).await?;
                return Ok(());
            }
            let template = modify_template(&ctx, &guild, bind_index, &args.change).await?;
            format!("`New Template`: {}", template)
        }
    };

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Success!")
        .description("The bind was successfully modified")
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

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

async fn add_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id};
    let index_str = format!("GroupBinds.{}.DiscordRoles", bind_index);
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
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id};
    let index_str = format!("GroupBinds.{}.DiscordRoles", bind_index);
    let update = doc! {"$pullAll": {index_str: role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_index: usize,
    template: &'t str,
) -> Result<String, RoError> {
    let template = match template {
        "N/A" => "{roblox-username}".into(),
        "disable" => "{discord-name}".into(),
        _ => template.to_string(),
    };
    let filter = doc! {"_id": guild.id};
    let index_str = format!("GroupBinds.{}.Template", bind_index);
    let update = doc! {"$set": {index_str: template.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(template)
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
            return Err(RoError::Argument(ArgumentError::ParseError {
                expected: "a number",
                usage: GroupbindsModifyArguments::generate_help(),
                name: "change",
            }));
        }
    };
    let filter = doc! {"_id": guild.id};
    let index_str = format!("GroupBinds.{}.Priority", bind_index);
    let update = doc! {"$set": {index_str: priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            "priority" => Ok(ModifyOption::Priority),
            _ => Err(ParseError(
                "one of `roles-add` `roles-remove` `template` `priority`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("Modify Groupbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
