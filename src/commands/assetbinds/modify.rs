use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static ASSETBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify", "m"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static ASSETBINDS_MODIFY_COMMAND: Command = Command {
    fun: assetbinds_modify,
    options: &ASSETBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn assetbinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
    };

    let asset_str = match args.next() {
        Some(a) => a.to_owned(),
        None => await_reply("Enter the ID of the asset to modify", ctx, msg).await?
    };
    let asset_id = match asset_str.parse::<i64>() {
        Ok(a) => a,
        Err(_) => return Err(RoError::Command(CommandError::ParseArgument(asset_str.into(), "Asset ID".into(), "Number".into())))
    };

    if !guild.assetbinds.iter().any(|g| g.id == asset_id) {
        return Ok(())
    }

    if field.eq_ignore_ascii_case("roles-add") {
        add_roles(ctx, &guild, asset_id, args).await?;
    } else if field.eq_ignore_ascii_case("roles-remove") {
        remove_roles(ctx, &guild, asset_id, args).await?;
    } 

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The bind was successfully modified").unwrap()
        .build().unwrap();

    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}

async fn add_roles(ctx: &Context, guild: &RoGuild, asset_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = bson::doc! {"$push": {"AssetBinds.$.DiscordRoles": {"$each": role_ids}}};
    ctx.database.modify_guild(filter, update).await
}

async fn remove_roles(ctx: &Context, guild: &RoGuild, asset_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "AssetBinds._id": asset_id};
    let update = bson::doc! {"$pullAll": {"AssetBinds.$.DiscordRoles": role_ids}};
    ctx.database.modify_guild(filter, update).await
}