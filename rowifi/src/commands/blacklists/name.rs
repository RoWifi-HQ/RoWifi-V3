use framework_new::prelude::*;
use mongodb::bson::{doc, to_bson};
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
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let username = args.username;
    let user_id = match ctx.bot.roblox.get_id_from_username(&username).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Blacklist Addition Failed")
                .unwrap()
                .description(format!(
                    "There was no user found with username {}",
                    username
                ))
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
    };

    let mut reason = args.reason;
    if reason.is_empty() {
        reason = "N/A".into();
    }

    let blacklist = Blacklist {
        id: user_id.to_string(),
        reason,
        blacklist_type: BlacklistType::Name(user_id.to_string()),
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
