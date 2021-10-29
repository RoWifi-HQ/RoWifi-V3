use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

#[derive(FromArgs)]
pub struct ModifyArguments {
    #[arg(
        help = "The field to modify. Must be one of `roles-add` `roles-remove` `priority` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The id of the asset to modify")]
    pub asset_id: i64,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    RolesAdd,
    RolesRemove,
    Priority,
    Template,
}

pub async fn assetbinds_modify(ctx: CommandContext, args: ModifyArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let field = args.option;
    let asset_id = args.asset_id;

    let bind = match guild.assetbinds.iter().find(|a| a.id == asset_id) {
        Some(a) => a,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Modification Failed")
                .description(format!("A bind with Asset Id {} does not exist", asset_id))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Success!")
        .description("The bind was successfully modified");
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Asset Bind Modification");
    let name = format!("Id: {}", asset_id);

    let desc = match field {
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, &guild, asset_id, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Added Roles: {}", modification);
            desc
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, &guild, asset_id, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Removed Roles: {}", modification);
            desc
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, &guild, asset_id, &args.change).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::Template => {
            if args.change.is_empty() {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .title("Asset Bind Modification Failed")
                    .description("You have entered a blank template")
                    .build()
                    .unwrap();
                ctx.respond().embeds(&[embed]).exec().await?;
                return Ok(());
            }
            let template = modify_template(&ctx, &guild, asset_id, &args.change).await?;
            format!("`New Template`: {}", template)
        }
    };

    let embed = embed
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed]).exec().await?;

    let log_embed = log_embed
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

async fn add_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    asset_id: i64,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = doc! {"$push": {"AssetBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    asset_id: i64,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = doc! {"$pullAll": {"AssetBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    guild: &RoGuild,
    asset_id: i64,
    template: &'t str,
) -> Result<String, RoError> {
    let template = match template {
        "N/A" => "{roblox-username}".into(),
        "disable" => "{discord-name}".into(),
        _ => template.to_string(),
    };
    let filter = doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = doc! {"$set": {"AssetBinds.$.Template": template.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(template)
}

async fn modify_priority(
    ctx: &CommandContext,
    guild: &RoGuild,
    asset_id: i64,
    priority: &str,
) -> Result<i64, RoError> {
    let priority = match priority.parse::<i64>() {
        Ok(p) => p,
        Err(_) => {
            return Err(RoError::Argument(ArgumentError::ParseError {
                expected: "a number",
                usage: ModifyArguments::generate_help(),
                name: "change",
            }));
        }
    };
    let filter = doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = doc! {"$set": {"AssetBinds.$.Priority": priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "priority" => Ok(ModifyOption::Priority),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `roles-add` `roles-remove` `template` `priority`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("Modify Assetbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
