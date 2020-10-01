use crate::framework::prelude::*;
use crate::models::guild::GuildType;

pub static PREMIUM_REDEEM_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["redeem"],
    desc: Some("Command to add premium to a server"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None
};

pub static PREMIUM_REMOVE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["remove"],
    desc: Some("Command to remove premium status of a server"),
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None
};


pub static PREMIUM_REDEEM_COMMAND: Command = Command {
    fun: premium_redeem,
    options: &PREMIUM_REDEEM_OPTIONS
};

pub static PREMIUM_REMOVE_COMMAND: Command = Command {
    fun: premium_remove,
    options: &PREMIUM_REMOVE_OPTIONS
};

#[command]
pub async fn premium_redeem(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let premium_user = match ctx.database.get_premium(msg.author.id.0).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                .title("Premium Redeem Failed").unwrap()
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details").unwrap()
                .build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
            return Ok(())
        }
    };

    let server = ctx.cache.guild(guild_id).unwrap();
    if !(server.owner_id == msg.author.id || ctx.config.owners.contains(&msg.author.id)) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Premium Redeem Failed").unwrap()
            .description("You must be the server owner to redeem premium in a server").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    }

    let guild_type: GuildType = premium_user.premium_type.into();
    match guild_type {
        GuildType::Alpha => {
            if premium_user.discord_servers.len() >= 1 {
                let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                    .title("Premium Redeem Failed").unwrap()
                    .description("You may only use premium in one of your servers").unwrap()
                    .build().unwrap();
                let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
                return Ok(())
            }
        },
        _ => {}
    };

    let _ = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let filter = bson::doc! {"_id": guild_id.0};
    let update = bson::doc! {"$set": {"Settings.Type": guild_type as i32, "Settings.AutoDetection": true}};
    ctx.database.modify_guild(filter, update).await?;

    if !premium_user.discord_servers.contains(&(guild_id.0 as i64)) {
        let filter2 = bson::doc! {"_id": msg.author.id.0};
        let update2 = bson::doc! {"$push": { "Servers": guild_id.0 }};
        ctx.database.modify_premium(filter2, update2).await?;
    }

    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Premium Redeem Successful").unwrap()
        .description(format!("Added Premium Features to {}", server.name)).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[command]
pub async fn premium_remove(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let premium_user = match ctx.database.get_premium(msg.author.id.0).await? {
        Some(p) => p,
        None => {
            let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
                .title("Premium Disable Failed").unwrap()
                .description("Premium Details corresponding to your account were not found. Please use `premium patreon` to link your details").unwrap()
                .build().unwrap();
            let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
            return Ok(())
        }
    };

    if !premium_user.discord_servers.contains(&(guild_id.0 as i64)) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Premium Disable Failed").unwrap()
            .description("This server either does not have premium enabled or the premium is owned by an another member").unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
        return Ok(())
    }

    let _ = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let filter = bson::doc! {"_id": guild_id.0};
    let update = bson::doc! {"$set": {"Settings.Type": GuildType::Normal as i32, "Settings.AutoDetection": false}};
    ctx.database.modify_guild(filter, update).await?;

    let filter2 = bson::doc! {"_id": msg.author.id.0};
    let update2 = bson::doc! {"$pull": { "Servers": guild_id.0 }};
    ctx.database.modify_premium(filter2, update2).await?;

    let server = ctx.cache.guild(guild_id).unwrap();
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Premium Disable Successful").unwrap()
        .description(format!("Removed Premium Features from {}", server.name)).unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    
    Ok(())
}