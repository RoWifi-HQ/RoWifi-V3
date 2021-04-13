use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::blacklist::{Blacklist, BlacklistType};

#[derive(FromArgs)]
pub struct BlacklistGroupArguments {
    #[arg(help = "The Group ID to blacklist")]
    pub group_id: i64,
    #[arg(help = "The reason of the blacklist", rest)]
    pub reason: String,
}

pub async fn blacklist_group(ctx: CommandContext, args: BlacklistGroupArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let group_id = args.group_id;
    let mut reason = args.reason;
    if reason.is_empty() {
        reason = "N/A".into();
    }
    let blacklist = Blacklist {
        id: group_id.to_string(),
        reason,
        blacklist_type: BlacklistType::Group(group_id),
    };
    let blacklist_bson = to_bson(&blacklist)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Blacklists": blacklist_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Blacklist Addition Successful")
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()).unwrap())
        .color(Color::DarkGreen as u32)
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Blacklist Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
