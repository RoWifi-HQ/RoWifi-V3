use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::RoGuild,
    rolang::{RoCommand, RoCommandUser},
};
use twilight_model::id::GuildId;

#[derive(FromArgs)]
pub struct CustombindsModifyArguments {
    #[arg(
        help = "The field to modify. Must be one of `code` `prefix` `priority` `roles-add` roles-remove` `template`"
    )]
    pub option: ModifyOption,
    #[arg(help = "The ID of the bind to modify")]
    pub id: i64,
    #[arg(help = "The actual modification to be made", rest)]
    pub change: String,
}

pub enum ModifyOption {
    Code,
    Prefix,
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
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let field = args.option;
    let bind_id = args.id;
    let bind = match guild.custombinds.iter().find(|c| c.id == bind_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Custom Bind Modification Failed")
                .unwrap()
                .description(format!("There was no bind found with id {}", bind_id))
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

    let name = format!("Id: {}", bind_id);
    let desc = match field {
        ModifyOption::Code => {
            let new_code = match modify_code(&ctx, &guild, bind_id, &args.change).await? {
                Some(n) => n,
                None => return Ok(()),
            };
            format!("`New Code`: {}", new_code)
        }
        ModifyOption::Prefix => {
            let new_prefix = modify_prefix(&ctx, &guild, bind_id, &args.change).await?;
            format!(
                "`Prefix`: {} -> {}",
                bind.prefix.as_ref().map_or("None", |s| s.as_str()),
                new_prefix
            )
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
            let template = modify_template(&ctx, &guild, bind_id, &args.change).await?;
            format!("`New Template`: {}", template)
        }
    };

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
        .description("Custom Bind Modification")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
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
                .unwrap()
                .title("Custom Bind Modification Failed")
                .unwrap()
                .description("You must be verified to create a custom blacklist")
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(None);
        }
    };
    let member = ctx
        .member(ctx.guild_id.unwrap(), ctx.author.id.0)
        .await?
        .unwrap();
    let ranks = ctx.bot.roblox.get_user_roles(user.roblox_id).await?;
    let username = ctx.bot.roblox.get_username_from_id(user.roblox_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &username,
    };
    let command = match RoCommand::new(code) {
        Ok(c) => c,
        Err(s) => {
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .content(s)
                .unwrap()
                .await?;
            return Ok(None);
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .content(res)
            .unwrap()
            .await?;
        return Ok(None);
    }
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Code": code}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(Some(code))
}

async fn modify_prefix(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    prefix: &str,
) -> Result<String, RoError> {
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Prefix": prefix}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(prefix.to_string())
}

async fn modify_template<'t>(
    ctx: &CommandContext,
    guild: &RoGuild,
    bind_id: i64,
    template: &'t str,
) -> Result<&'t str, RoError> {
    let filter = doc! {"_id": guild.id, "CustomBinds._id": bind_id};
    let update = doc! {"$set": {"CustomBinds.$.Template": template}};
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
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
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
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
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
            "prefix" => Ok(ModifyOption::Prefix),
            "priority" => Ok(ModifyOption::Priority),
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            "template" => Ok(ModifyOption::Template),
            _ => Err(ParseError(
                "one of `code` `prefix` `priority` `roles-add` `roles-remove` `template`",
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
