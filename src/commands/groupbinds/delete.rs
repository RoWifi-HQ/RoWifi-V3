use crate::framework::prelude::*;

pub static GROUPBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["delete", "d"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static GROUPBINDS_DELETE_COMMAND: Command = Command {
    fun: groupbinds_delete,
    options: &GROUPBINDS_DELETE_OPTIONS
};

#[command]
pub async fn groupbinds_delete(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let mut groups_to_delete = Vec::new();
    while let Some(arg) = args.next() {
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

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"GroupBinds": {"GroupId": {"$in": binds_to_delete}}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given bind were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}