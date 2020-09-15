use crate::framework::prelude::*;

pub static CUSTOMBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
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
    let update = bson::doc! {"$pull": {"CustomBinds": {"_id": {"$in": binds_to_delete}}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given bind were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}