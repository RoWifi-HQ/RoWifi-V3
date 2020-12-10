use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

pub static ASSETBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify", "m"],
    desc: Some("Command to modify an asset bind"),
    usage: Some(
        "assetbinds modify <Field> <Asset Id> [Roles..]`\nField: `roles-add` `roles-remove",
    ),
    examples: &[
        "assetbinds modify roles-add 8998774 @Role1 @Role2",
        "ab m roles-remove 8998774 @Role1",
    ],
    min_args: 3,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static ASSETBINDS_MODIFY_COMMAND: Command = Command {
    fun: assetbinds_modify,
    options: &ASSETBINDS_MODIFY_OPTIONS,
};

#[command]
pub async fn assetbinds_modify(
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

    let asset_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => {
                return Err(CommandError::ParseArgument(
                    a.into(),
                    "Asset ID".into(),
                    "Number".into(),
                )
                .into())
            }
        },
        None => return Ok(()),
    };

    if !guild.assetbinds.iter().any(|a| a.id == asset_id) {
        let e = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Asset Modification Failed")
            .unwrap()
            .description(format!("A bind with Asset Id {} does not exist", asset_id))
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(e)
            .unwrap()
            .await?;
        return Ok(());
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Success!")
        .unwrap()
        .description("The bind was successfully modified")
        .unwrap();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Asset Bind Modification")
        .unwrap();
    let name = format!("Id: {}", asset_id);
    let desc = if field.eq_ignore_ascii_case("roles-add") {
        let role_ids = add_roles(ctx, &guild, asset_id, args).await?;

        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        let desc = format!("Added Roles: {}", modification);
        desc
    } else if field.eq_ignore_ascii_case("roles-remove") {
        let role_ids = remove_roles(ctx, &guild, asset_id, args).await?;

        let modification = role_ids
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>();
        let desc = format!("Removed Roles: {}", modification);
        desc
    } else {
        return Ok(());
    };

    let embed = embed
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    let log_embed = log_embed
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger
        .log_guild(ctx, msg.guild_id.unwrap(), log_embed)
        .await;

    Ok(())
}

async fn add_roles(
    ctx: &Context,
    guild: &RoGuild,
    asset_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = bson::doc! {"$push": {"AssetBinds.$.DiscordRoles": {"$each": role_ids.clone()}}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}

async fn remove_roles(
    ctx: &Context,
    guild: &RoGuild,
    asset_id: i64,
    args: Arguments<'_>,
) -> Result<Vec<u64>, RoError> {
    let mut role_ids = Vec::new();
    for r in args {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = bson::doc! {"$pullAll": {"AssetBinds.$.DiscordRoles": role_ids.clone()}};
    ctx.database.modify_guild(filter, update).await?;
    Ok(role_ids)
}
