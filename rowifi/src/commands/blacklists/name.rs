use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::blacklist::{Blacklist, BlacklistType};

#[derive(FromArgs)]
pub struct BlacklistNameArguments {
    #[arg(help = "The username to blacklist. This will get converted into the id in the database")]
    pub username: String,
    #[arg(help = "The reason of the blacklist", rest)]
    pub reason: String,
}

pub async fn blacklist_name(ctx: CommandContext, args: BlacklistNameArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let username = args.username;
    let user = match ctx.bot.roblox.get_user_from_username(&username).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Blacklist Addition Failed")
                .description(format!(
                    "There was no user found with username {}",
                    username
                ))
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let mut reason = args.reason;
    if reason.is_empty() {
        reason = "N/A".into();
    }

    let blacklist = Blacklist {
        id: user.id.0.to_string(),
        reason,
        blacklist_type: BlacklistType::Name(user.id.0.to_string()),
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
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .color(Color::DarkGreen as u32)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Blacklist Addition")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
