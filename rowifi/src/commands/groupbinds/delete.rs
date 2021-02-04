use framework_new::prelude::*;
use mongodb::bson::doc;

#[derive(FromArgs)]
pub struct GroupbindsDeleteArguments {
    #[arg(help = "The ID of the groupbind to delete", rest)]
    pub id: String,
}

pub async fn groupbinds_delete(
    ctx: CommandContext,
    args: GroupbindsDeleteArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let mut groups_to_delete = Vec::new();
    for arg in args.id.split_ascii_whitespace() {
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"GroupBinds": {"GroupId": {"$in": binds_to_delete.clone()}}}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(e)
        .unwrap()
        .await?;

    let ids_str = binds_to_delete
        .iter()
        .map(|b| format!("`Group Id`: {}\n", b))
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
