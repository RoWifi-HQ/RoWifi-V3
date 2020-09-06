use crate::framework::prelude::*;
use itertools::Itertools;

pub static BLACKLISTS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["delete", "d"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static BLACKLISTS_DELETE_COMMAND: Command = Command {
    fun: blacklists_delete,
    options: &BLACKLISTS_DELETE_OPTIONS
};

#[command]
pub async fn blacklists_delete(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let id = args.join(" ");

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"Blacklists": {"_id": id}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given bind were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}