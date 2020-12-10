use rowifi_framework::prelude::*;
use itertools::Itertools;

pub static BLACKLISTS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["delete", "d", "remove"],
    desc: Some("Command to delete a blacklist"),
    usage: Some("blacklists delete <Id>"),
    examples: &[
        "blacklists delete 3108077",
        "bl delete 3108077",
        "blacklists d IsInGroup(3108077)",
    ],
    min_args: 1,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static BLACKLISTS_DELETE_COMMAND: Command = Command {
    fun: blacklists_delete,
    options: &BLACKLISTS_DELETE_OPTIONS,
};

#[command]
pub async fn blacklists_delete(
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

    let id = args.join(" ");
    let blacklist = guild.blacklists.iter().find(|b| b.id == id);
    if blacklist.is_none() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Blacklist Deletion Failed")
            .unwrap()
            .description("A blacklist with the given id was not found")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"Blacklists": {"_id": id}}};
    ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Blacklist Deletion Successful")
        .unwrap()
        .description("The given blacklist was successfully deleted")
        .unwrap()
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let blacklist = blacklist.unwrap();
    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Blacklist Deletion")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
