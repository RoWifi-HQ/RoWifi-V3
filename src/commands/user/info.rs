use rowifi_framework::prelude::*;
use twilight_embed_builder::{EmbedFieldBuilder, ImageSource};
use twilight_model::id::{GuildId, UserId};

pub static USERINFO_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["userinfo"],
    desc: Some("Command to view the information about an user"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous"),
};

pub static USERINFO_COMMAND: Command = Command {
    fun: userinfo,
    options: &USERINFO_OPTIONS,
};

pub static BOTINFO_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["botinfo"],
    desc: Some("Command to view the information about the bot"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous"),
};

pub static BOTINFO_COMMAND: Command = Command {
    fun: botinfo,
    options: &BOTINFO_OPTIONS,
};

pub static SUPPORT_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["support", "invite"],
    desc: Some("Command to view the supporting links for the bot"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous"),
};

pub static SUPPORT_COMMAND: Command = Command {
    fun: support,
    options: &SUPPORT_OPTIONS,
};

#[command]
pub async fn userinfo(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let author = match args
        .next()
        .and_then(parse_username)
        .and_then(|u| ctx.cache.user(UserId(u)))
    {
        Some(u) => (u.id, u.name.to_owned()),
        None => (msg.author.id, msg.author.name.to_owned()),
    };
    let user = match ctx.database.get_user((author.0).0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("User Info Failed")
                .unwrap()
                .description("User was not verified. Please ask him/her to verify themselves")
                .unwrap()
                .build()
                .unwrap();
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await;
            return Ok(());
        }
    };

    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title(author.1.clone())
        .unwrap()
        .description("Profile Information")
        .unwrap()
        .field(EmbedFieldBuilder::new("Username", username.clone()).unwrap())
        .field(EmbedFieldBuilder::new("Roblox Id", user.roblox_id.to_string()).unwrap())
        .field(EmbedFieldBuilder::new("Discord Id", user.discord_id.to_string()).unwrap())
        .thumbnail(
            ImageSource::url(format!(
                "http://www.roblox.com/Thumbs/Avatar.ashx?x=150&y=150&Format=Png&username={}",
                username
            ))
            .unwrap(),
        )
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[command]
pub async fn botinfo(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let current_user = ctx.cache.current_user().unwrap();
    let guilds = ctx.cache.guilds();
    let member_count: i64 = guilds
        .iter()
        .map(|g| ctx.cache.member_count(GuildId(*g)))
        .sum();

    let embed = EmbedBuilder::new()
        .default_data()
        .field(
            EmbedFieldBuilder::new(
                "Name",
                format!("{}#{}", current_user.name, current_user.discriminator),
            )
            .unwrap()
            .inline(),
        )
        .field(EmbedFieldBuilder::new("Version", "2.7.1").unwrap().inline())
        .field(EmbedFieldBuilder::new("Language", "Rust").unwrap().inline())
        .field(
            EmbedFieldBuilder::new("Shards", ctx.bot_config.total_shards.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Cluster Id", ctx.bot_config.cluster_id.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Servers", guilds.len().to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Members", member_count.to_string())
                .unwrap()
                .inline(),
        )
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[command]
pub async fn support(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let disc_link = "https://www.discord.gg/h4BGGyR";
    let invite_link = "https://discordapp.com/oauth2/authorize?client_id=508968886998269962&scope=bot&permissions=402672704";
    let website = "https://rowifi.link";
    let embed = EmbedBuilder::new()
        .default_data()
        .field(
            EmbedFieldBuilder::new(
                "Support Server",
                format!(
                    "To know more about announcements, updates and other stuff: [Click Here]({})",
                    disc_link
                ),
            )
            .unwrap(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Invite Link",
                format!(
                    "To invite the bot into your server: [Click Here]({})",
                    invite_link
                ),
            )
            .unwrap(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Website",
                format!("To check out our website: [Click Here]({})", website),
            )
            .unwrap(),
        )
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
