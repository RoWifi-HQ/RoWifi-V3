use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Groupbind},
    id::RoleId,
};

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
    let groupbinds = ctx
        .bot
        .database
        .query::<Groupbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id",
            &[&(guild_id), &BindType::Group],
        )
        .await?;

    let field = args.option;
    let group_id = args.group_id;
    let bind = match groupbinds.iter().find(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Group Bind Modification Failed")
                .description(format!("There was no bind found with id {}", group_id))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let name = format!("Id: {}", group_id);
    let desc = match field {
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Added Roles: {}", modification);
            desc
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            let desc = format!("Removed Roles: {}", modification);
            desc
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, bind, &args.change).await?;
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
                ctx.respond().embeds(&[embed])?.exec().await?;
                return Ok(());
            }
            let template = modify_template(&ctx, bind, &args.change).await?;
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
    ctx.respond().embeds(&[embed])?.exec().await?;

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
    bind: &Groupbind,
    roles: &str,
) -> Result<Vec<RoleId>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    ctx.bot.database.execute("UPDATE binds SET discord_roles = array_cat(discord_roles, $1::BIGINT[]) WHERE bind_id = $2", &[&role_ids, &bind.bind_id]).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    bind: &Groupbind,
    roles: &str,
) -> Result<Vec<RoleId>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    let mut roles_to_keep = bind.discord_roles.clone();
    roles_to_keep.retain(|r| !role_ids.contains(r));
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET discord_roles = $1 WHERE bind_id = $2",
            &[&roles_to_keep, &bind.bind_id],
        )
        .await?;
    Ok(role_ids)
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    bind: &Groupbind,
    template: &'t str,
) -> Result<String, RoError> {
    let template = match template {
        "N/A" => "{roblox-username}".into(),
        "disable" => "{discord-name}".into(),
        _ => template.to_string(),
    };
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET template = $1 WHERE bind_id = $2",
            &[&template, &bind.bind_id],
        )
        .await?;
    Ok(template)
}

async fn modify_priority(
    ctx: &CommandContext,
    bind: &Groupbind,
    priority: &str,
) -> Result<i32, RoError> {
    let priority = match priority.parse::<i32>() {
        Ok(p) => p,
        Err(_) => {
            return Err(ArgumentError::ParseError {
                expected: "a number",
                usage: GroupbindsModifyArguments::generate_help(),
                name: "change",
            }
            .into());
        }
    };
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET priority = $1 WHERE bind_id = $2",
            &[&priority, &bind.bind_id],
        )
        .await?;
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
        let arg = match &option.value {
            CommandOptionValue::String(value) => value.to_string(),
            CommandOptionValue::Integer(value) => value.to_string(),
            _ => unreachable!("Modify Groupbinds unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
