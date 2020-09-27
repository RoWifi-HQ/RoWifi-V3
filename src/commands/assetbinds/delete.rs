use crate::framework::prelude::*;

pub static ASSETBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["delete", "remove", "d"],
    desc: Some("Command to delete assetbinds"),
    usage: Some("assetbinds delete <Id>"),
    examples: &["assetbinds delete 792688917", "ab remove 792688917"],
    required_permissions: Permissions::empty(),
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static ASSETBINDS_DELETE_COMMAND: Command = Command {
    fun: assetbinds_delete,
    options: &ASSETBINDS_DELETE_OPTIONS
};

#[command]
pub async fn assetbinds_delete(ctx: &Context, msg: &Message, args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let mut assets_to_delete = Vec::new();
    for arg in args {
        if let Ok(r) = arg.parse::<i64>() {
            assets_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for asset in assets_to_delete {
        if let Some(b) = guild.assetbinds.iter().find(|r| r.id == asset) {
            binds_to_delete.push(b.id);
        }
    }

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"AssetBinds": {"_id": {"$in": binds_to_delete.clone()}}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given binds were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;

    let ids_str = binds_to_delete.iter().map(|b| format!("`Id`: {}\n", b)).collect::<String>();
    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description("Asset Bind Deletion").unwrap()
        .field(EmbedFieldBuilder::new("Assets Deleted", ids_str).unwrap()).build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    
    Ok(())
}