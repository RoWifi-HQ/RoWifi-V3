use mongodb::bson::doc;
use rowifi_framework::prelude::*;

#[derive(FromArgs)]
pub struct BlacklistDeleteArguments {
    #[arg(help = "The ID of the blacklist to delete", rest)]
    pub id: String,
}

pub async fn blacklist_delete(
    ctx: CommandContext,
    args: BlacklistDeleteArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let id = args.id;
    let blacklist = guild.blacklists.iter().find(|b| b.id == id);
    if blacklist.is_none() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Blacklist Deletion Failed")
            .description("A blacklist with the given id was not found")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"Blacklists": {"_id": id}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Blacklist Deletion Successful")
        .description("The given blacklist was successfully deleted")
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let blacklist = blacklist.unwrap();
    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Blacklist Deletion")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
