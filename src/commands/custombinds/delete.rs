use crate::framework::prelude::*;

pub static CUSTOMBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["delete", "d", "remove"],
    desc: Some("Command to delete a custombind"),
    usage: Some("custombinds delete <Id>"),
    examples: &["custombinds delete 1", "cb remove 2"],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static CUSTOMBINDS_DELETE_COMMAND: Command = Command {
    fun: custombinds_delete,
    options: &CUSTOMBINDS_DELETE_OPTIONS
};

#[command]
pub async fn custombinds_delete(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let mut ids_to_delete = Vec::new();
    while let Some(arg) = args.next() {
        if let Ok(r) = arg.parse::<i64>() {
            ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for id in ids_to_delete {
        if guild.custombinds.iter().find(|r| r.id == id).is_some() {
            binds_to_delete.push(id);
        }
    }

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"CustomBinds": {"_id": {"$in": binds_to_delete.clone()}}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given binds were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;

    let ids_str = binds_to_delete.iter().map(|b| format!("`Id`: {}\n", b)).collect::<String>();
    let log_embed = EmbedBuilder::new().default_data()
        .title(format!("Action by {}", msg.author.name)).unwrap()
        .description("Custom Bind Deletion").unwrap()
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str).unwrap()).build().unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}