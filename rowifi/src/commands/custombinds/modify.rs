use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::RoGuild,
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;
use twilight_model::id::GuildId;

#[derive(FromArgs)]
pub struct CustombindsModifyArguments {
    #[arg(
        help = "The field to modify. Must be one of `code` `priority` `roles-add` `roles-remove` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The ID of the bind to modify")]
    pub id: i64,
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
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let field = args.option;
    let bind_id = args.id;
    let bind = match guild.custombinds.iter().find(|c| c.id == bind_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Modification Failed")
                .description(format!("There was no bind found with id {}", bind_id))
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let name = format!("Id: {}", bind_id);
    let desc = match field {
        ModifyOption::Code => {
            let new_code = match modify_code(&ctx, &guild, bind_id, &args.change).await? {
                Some(n) => n,
                None => return Ok(()),
            };
            format!("`New Code`: {}", new_code)
        }
        ModifyOption::Priority => {
            let new_priority = modify_priority(&ctx, &guild, bind_id, &args.change).await?;
            format!("`Priority`: {} -> {}", bind.priority, new_priority)
        }
        ModifyOption::RolesAdd => {
            let role_ids = add_roles(&ctx, &guild, bind_id, &args.change).await?;
            let modification = role_ids
                .iter()
                .map(|r| format!("<@&{}> ", r))
                .collect::<String>();
            format!("Added Roles: {}", modification)
        }
        ModifyOption::RolesRemove => {
            let role_ids = remove_roles(&ctx, &guild, bind_id, &args.change).await?;
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
                ctx.respond().embed(embed).await?;
                return Ok(());
            }
            let template = modify_template(&ctx, &guild, bind_id, &args.change).await?;
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
        .description("Custom Bind Modification")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

async fn modify_code<'a>(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    code: &'a str,
) -> Result<Option<&'a str>, RoError> {
    let user = match ctx
        .get_linked_user(ctx.author.id, GuildId(guild.id as u64))
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
            ctx.respond().embed(embed).await?;
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
    let roblox_user = ctx.bot.roblox.get_user(user_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(s).await?;
            return Ok(None);
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(res).await?;
        return Ok(None);
    }
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Code": code}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(Some(code))
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    template: &'t str,
) -> Result<String, RoError> {
    let template = match template {
        "N/A" => "{roblox-username}".into(),
        "disable" => "{discord-name}".into(),
        _ => template.to_string(),
    };
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Template": template.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(template)
}

async fn modify_priority(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    priority: &str,
) -> Result<i64, RoError> {
    let priority = match priority.parse::<i64>() {
        Ok(p) => p,
        Err(_) => {
            return Err(RoError::Argument(ArgumentError::ParseError {
                expected: "a number",
                usage: CustombindsModifyArguments::generate_help(),
                name: "change",
            }));
        }
    };
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Priority": priority}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(priority)
}

async fn add_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$push": {"CustomBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    roles: &str,
) -> Result<Vec<i64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r as i64);
        }
    }
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$pullAll": {"CustomBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
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
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("ModifyArgument unreached"),
        };

        ModifyOption::from_arg(&arg)
    }
}
