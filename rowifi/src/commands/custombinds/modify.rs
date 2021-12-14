use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{BindType, Custombind},
    discord::id::GuildId,
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

#[derive(FromArgs)]
pub struct CustombindsModifyArguments {
    #[arg(
        help = "The field to modify. Must be one of `code` `priority` `roles-add` `roles-remove` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The ID of the bind to modify")]
    pub id: i32,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    Code,
    Priority,
    RolesAdd,
    RolesRemove,
    Template,
}

pub async fn custombinds_modify(
    ctx: CommandContext,
    args: CustombindsModifyArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id.get() as i64), &BindType::Custom],
        )
        .await?;

    let field = args.option;
    let id_to_modify = args.id;
    let bind = match custombinds
        .iter()
        .find(|c| c.custom_bind_id == id_to_modify)
    {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", id_to_modify))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let name = format!("Id: {}", id_to_modify);
    let desc = match field {
        ModifyOption::Code => {
            let new_code = match modify_code(&ctx, guild_id, bind, &args.change).await? {
                Some(n) => n,
                None => return Ok(()),
            };
            format!("`New Code`: {}", new_code)
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, bind, &args.change).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Added Roles: {}", modification)
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, bind, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Removed Roles: {}", modification)
        }
        ModifyOption::Template => {
            if args.change.is_empty() {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .color(Color::Red as u32)
                    .title("Custom Bind Modification Failed")
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
        .description("Custom Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

async fn modify_code<'a>(
    ctx: &CommandContext,
    guild_id: GuildId,
    bind: &Custombind,
    code: &'a str,
) -> Result<Option<&'a str>, RoError> {
    let user = match ctx
        .bot
        .database
        .get_linked_user(ctx.author.id.get() as i64, guild_id.get() as i64)
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description("You must be verified to create a custom blacklist")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(None);
        }
    };

    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx
        .member(ctx.guild_id.unwrap(), ctx.author.id.0)
        .await?
        .unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id, false).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(&s)?.exec().await?;
            return Ok(None);
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(&res)?.exec().await?;
        return Ok(None);
    }
    ctx.bot
        .database
        .execute(
            "UPDATE binds SET code = $1 WHERE bind_id = $2",
            &[&code, &bind.bind_id],
        )
        .await?;
    Ok(Some(code))
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    bind: &Custombind,
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
    bind: &Custombind,
    priority: &str,
) -> Result<i64, RoError> {
    let priority = match priority.parse::<i64>() {
        Ok(p) => p,
        Err(_) => {
            return Err(ArgumentError::ParseError {
                expected: "a number",
                usage: CustombindsModifyArguments::generate_help(),
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

async fn add_roles(
    ctx: &CommandContext,
    bind: &Custombind,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| r.0.get() as i64));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();
    ctx.bot.database.execute("UPDATE binds SET discord_roles = array_cat(discord_roles, $1::BIGINT[]) WHERE bind_id = $2", &[&role_ids, &bind.bind_id]).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    bind: &Custombind,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| r.0.get() as i64));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
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

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "code" => Ok(ModifyOption::Code),
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `code` `priority` `roles-add` `roles-remove` `template`",
            )),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match &option.value {
            CommandOptionValue::String(value) => value.to_string(),
            CommandOptionValue::Integer(value) => value.to_string(),
            _ => unreachable!("ModifyArgument unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
