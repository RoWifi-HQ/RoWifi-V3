use rowifi_framework::prelude::*;

pub static GROUPBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["delete", "d", "remove"],
    desc: Some("Command to delete a groupbind"),
    usage: Some("groupbinds delete <Group Id>"),
    examples: &["groupbinds delete 3108077", "gb remove 5581309"],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static GROUPBINDS_DELETE_COMMAND: Command = Command {
    fun: groupbinds_delete,
    options: &GROUPBINDS_DELETE_OPTIONS,
};

#[command]
pub async fn groupbinds_delete(
    ctx: &Context,
    msg: &Message,
    args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let mut groups_to_delete = Vec::new();
    for arg in args {
        if let Ok(r) = arg.parse::<i64>() {
            groups_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for group in groups_to_delete {
        if let Some(b) = guild.groupbinds.iter().find(|r| r.group_id == group) {
            binds_to_delete.push(b.group_id);
        }
    }

    if binds_to_delete.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Binds Deletion Failed")
            .unwrap()
            .description("There were no binds found associated with given ids")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await;
        return Ok(());
    }

    let filter = bson::doc! {"_id": guild.id};
    let update =
        bson::doc! {"$pull": {"GroupBinds": {"GroupId": {"$in": binds_to_delete.clone()}}}};
    ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Success!")
        .unwrap()
        .description("The given binds were successfully deleted")
        .unwrap()
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let ids_str = binds_to_delete
        .iter()
        .map(|b| format!("`Group Id`: {}\n", b))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Custom Bind Deletion")
        .unwrap()
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
