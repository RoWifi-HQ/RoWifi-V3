use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static GROUPBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify", "m"],
    desc: Some("Command to modify a groupbind"),
    usage: Some(
        "groupbinds modify <Field> <Group Id> [Roles..]`\nField: `roles-add` `roles-remove",
    ),
    examples: &[
        "groupbinds modify roles-add 8998774 @Role1 @Role2",
        "gb m roles-remove 8998774 @Role1",
    ],
    required_permissions: Permissions::empty(),
    min_args: 3,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static GROUPBINDS_MODIFY_COMMAND: Command = Command {
    fun: groupbinds_modify,
    options: &GROUPBINDS_MODIFY_OPTIONS,
};

#[command]
pub async fn groupbinds_modify(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(()),
    };

    let group_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => {
                return Err(CommandError::ParseArgument(
                    a.into(),
                    "Group ID".into(),
                    "Number".into(),
                )
                .into())
            }
        },
        None => return Ok(()),
    };

    if guild.groupbinds.iter().any(|c| c.group_id == group_id) {
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
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap();
        return Ok(());
    };

    let name = format!("Id: {}", group_id);
    let desc = if field.eq_ignore_ascii_case("roles-add") {
        let role_ids = add_roles(ctx, &guild, group_id, args).await?;
        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        let desc = format!("Added Roles: {}", modification);
        desc
    } else if field.eq_ignore_ascii_case("roles-remove") {
        let role_ids = remove_roles(ctx, &guild, group_id, args).await?;
        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        let desc = format!("Removed Roles: {}", modification);
        desc
    } else {
        return Err(CommandError::ParseArgument(
            field,
            "Field".into(),
            "`roles-add`, `roles-remove`".into(),
        )
        .into());
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
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Group Bind Modification")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}

async fn add_roles(
    ctx: &Context,
    guild: &RoGuild,
    group_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "GroupBinds.GroupId": group_id};
    let update = bson::doc! {"$push": {"GroupBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &Context,
    guild: &RoGuild,
    group_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "GroupBinds.GroupId": group_id};
    let update = bson::doc! {"$pullAll": {"GroupBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}
