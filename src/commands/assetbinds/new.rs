use crate::framework::prelude::*;
use crate::models::bind::{AssetBind, AssetType};
use twilight_mention::Mention;

pub static ASSETBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to add a new asset bind"),
    usage: Some("assetbinds new <Field> <Asset Id> [Roles..]`\n`Field`: `Asset` `Badge` `Gamepass"),
    examples: &["assetbinds new Asset 78978292 @Role1", "ab new Gamepass 79820839 @Role2", "assetbinds new Badge 8799292 @Role1 @Role2"],
    required_permissions: Permissions::empty(),
    hidden: false,
    min_args: 3,
    sub_commands: &[],
    group: None
};

pub static ASSETBINDS_NEW_COMMAND: Command = Command {
    fun: assetbinds_new,
    options: &ASSETBINDS_NEW_OPTIONS
};

#[command]
pub async fn assetbinds_new(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let asset_type = match args.next() {
        Some(a) => match a.parse::<AssetType>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Asset Type".into(), "Asset, Badge, Gamepass".into()).into())
        },
        None => return Ok(())
    };

    let asset_id = match args.next() {
        Some(a) => match a.parse::<i64>() {
            Ok(a) => a,
            Err(_) => return Err(CommandError::ParseArgument(a.into(), "Asset ID".into(), "Number".into()).into())
        },
        None => return Ok(())
    };

    if guild.assetbinds.iter().any(|a| a.asset_type == asset_type && a.id == asset_id) {
        let embed = EmbedBuilder::new().default_data().title("Bind Addition Failed").unwrap()
            .color(Color::Red as u32).unwrap()
            .description(format!("A bind with asset id {} already exists", asset_id)).unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    }

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let mut roles: Vec<i64> = Vec::new();
    for r in args {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }
    if roles.is_empty() {
        let embed = EmbedBuilder::new().default_data().title("Bind Addition Failed").unwrap()
            .color(Color::Red as u32).unwrap()
            .description("Atleast role must be entered to create an assetbind").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    } 

    let bind = AssetBind {id: asset_id, asset_type, discord_roles: roles};
    let bind_bson = bson::to_bson(&bind)?;

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"AssetBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", asset_id);
    let value = format!("Type: {}\nRoles: {}", bind.asset_type, bind.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>());
    let embed = EmbedBuilder::new().default_data().title("Bind Addition Successful").unwrap()
        .color(Color::DarkGreen as u32).unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), value.clone()).unwrap())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;

    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description("Asset Bind Addition").unwrap()
        .field(EmbedFieldBuilder::new(name, value).unwrap()).build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}