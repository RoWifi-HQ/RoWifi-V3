use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

#[derive(FromArgs)]
pub struct GroupbindsModifyArguments {
    #[arg(help = "The field to modify. Must be either `roles-add` or `roles-remove`")]
    pub option: ModifyOption,
    #[arg(help = "The id of the groupbind to modify")]
    pub group_id: i64,
    #[arg(help = "The actual modification to be made")]
    pub change: String,
}

pub enum ModifyOption {
    RolesAdd,
    RolesRemove,
}

pub async fn groupbinds_modify(
    ctx: CommandContext,
    args: GroupbindsModifyArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let field = args.option;
    let group_id = args.group_id;
    let bind_index = match guild.groupbinds.iter().position(|g| g.group_id == group_id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Group Bind Modification Failed")
                .unwrap()
                .description(format!("There was no bind found with id {}", group_id))
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap();
            return Ok(());
        }
    };

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
        .description("Group Bind Modification")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
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
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
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
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in roles.split_ascii_whitespace() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = doc! {"_id": guild.id};
    let index_str = format!("GroupBinds.{}.DiscordRoles", bind_index);
    let update = doc! {"$pullAll": {index_str: role_ids.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

impl FromArg for ModifyOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "roles-add" => Ok(ModifyOption::RolesAdd),
            "roles-remove" => Ok(ModifyOption::RolesRemove),
            _ => Err(ParseError("one of `roles-add` `roles-remove`")),
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
