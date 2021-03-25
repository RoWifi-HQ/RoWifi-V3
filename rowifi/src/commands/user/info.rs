use rowifi_framework::prelude::*;
use twilight_embed_builder::{EmbedFieldBuilder, ImageSource};
use twilight_model::id::{GuildId, UserId};

#[derive(FromArgs)]
pub struct UserInfoArguments {
    pub user: Option<UserId>,
}

pub async fn userinfo(ctx: CommandContext, args: UserInfoArguments) -> CommandResult {
    let author = match args.user.and_then(|u| ctx.bot.cache.user(u)) {
        Some(u) => (u.id, u.name.clone()),
        None => (ctx.author.id, ctx.author.name.clone()),
    };
    let user = match ctx.get_linked_user(author.0, ctx.guild_id.unwrap()).await? {
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
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let username = ctx.bot.roblox.get_username_from_id(user.roblox_id).await?;

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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct BotInfoArguments {}

pub async fn botinfo(ctx: CommandContext, _args: BotInfoArguments) -> CommandResult {
    let current_user = ctx.bot.cache.current_user().unwrap();
    let guilds = ctx.bot.cache.guilds();
    let member_count: i64 = guilds
        .iter()
        .map(|g| ctx.bot.cache.member_count(GuildId(*g)))
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
        .field(
            EmbedFieldBuilder::new("Version", env!("CARGO_PKG_VERSION"))
                .unwrap()
                .inline(),
        )
        .field(EmbedFieldBuilder::new("Language", "Rust").unwrap().inline())
        .field(
            EmbedFieldBuilder::new("Shards", ctx.bot.total_shards.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Cluster Id", ctx.bot.cluster_id.to_string())
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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct SupportArguments {}

pub async fn support(ctx: CommandContext, _args: SupportArguments) -> CommandResult {
    let disc_link = "https://www.discord.gg/h4BGGyR";
    let invite_link = "https://discord.com/oauth2/authorize?client_id=508968886998269962&scope=bot%20applications.commands&permissions=402672704";
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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
