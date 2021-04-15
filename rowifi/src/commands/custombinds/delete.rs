use mongodb::bson::doc;
use rowifi_framework::prelude::*;

#[derive(FromArgs)]
pub struct CustombindsDeleteArguments {
    #[arg(help = "The ID of the custombind to delete", rest)]
    pub id: String,
}

pub async fn custombinds_delete(
    ctx: CommandContext,
    args: CustombindsDeleteArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let mut ids_to_delete = Vec::new();
    for arg in args.id.split_ascii_whitespace() {
        if let Ok(r) = arg.parse::<i64>() {
            ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for id in ids_to_delete {
        if guild.custombinds.iter().any(|r| r.id == id) {
            binds_to_delete.push(id);
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
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"CustomBinds": {"_id": {"$in": binds_to_delete.clone()}}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Success!")
        .unwrap()
        .description("The given binds were successfully deleted")
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let ids_str = binds_to_delete
        .iter()
        .map(|b| format!("`Id`: {}\n", b))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Custom Bind Deletion")
        .unwrap()
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
