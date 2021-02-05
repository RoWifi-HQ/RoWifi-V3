use framework_new::prelude::*;

#[derive(FromArgs)]
pub struct RankBindsDelete {
    #[arg(help = "The Group ID of the Rankbind to delete")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the Rankbind to delete")]
    pub rank_id: String,
}

pub async fn rankbinds_delete(ctx: CommandContext, args: RankBindsDelete) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let group_id = args.group_id;

    let mut rank_ids_to_delete = Vec::new();
    for arg in args.rank_id.split_ascii_whitespace() {
        if let Ok(r) = arg.parse::<i64>() {
            rank_ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for rank in rank_ids_to_delete {
        if let Some(b) = guild
            .rankbinds
            .iter()
            .find(|r| r.group_id == group_id && r.rank_id == rank)
        {
            binds_to_delete.push(b.rbx_rank_id);
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

    let filter = bson::doc! {"_id": guild.id};
    let update =
        bson::doc! {"$pull": {"RankBinds": {"RbxGrpRoleId": {"$in": binds_to_delete.clone()}}}};
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
        .map(|b| format!("`Id`: {}\n", b))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Rank Bind Deletion")
        .unwrap()
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
