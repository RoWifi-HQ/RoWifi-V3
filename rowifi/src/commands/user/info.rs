use rowifi_framework::prelude::*;
use rowifi_models::roblox::id::UserId as RobloxUserId;
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
                .title("User Info Failed")
                .description("User was not verified. Please ask him/her to verify themselves")
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

    let roblox_user = ctx
        .bot
        .roblox
        .get_user(RobloxUserId(user.roblox_id as u64))
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title(author.1.clone())
        .description("Profile Information")
        .field(EmbedFieldBuilder::new("Username", roblox_user.name.clone()))
        .field(EmbedFieldBuilder::new("Roblox Id", user.roblox_id.to_string()))
        .field(EmbedFieldBuilder::new("Discord Id", user.discord_id.to_string()))
        .thumbnail(
            ImageSource::url(format!(
                "http://www.roblox.com/Thumbs/Avatar.ashx?x=150&y=150&Format=Png&username={}",
                roblox_user.name
            ))
            .unwrap(),
        )
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
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
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Version", env!("CARGO_PKG_VERSION")).inline(),
        )
        .field(EmbedFieldBuilder::new("Language", "Rust").inline())
        .field(
            EmbedFieldBuilder::new("Shards", ctx.bot.total_shards.to_string()).inline(),
        )
        .field(
            EmbedFieldBuilder::new("Cluster Id", ctx.bot.cluster_id.to_string()).inline(),
        )
        .field(
            EmbedFieldBuilder::new("Servers", guilds.len().to_string()).inline(),
        )
        .field(
            EmbedFieldBuilder::new("Members", member_count.to_string()).inline(),
        )
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct SupportArguments {}

pub async fn support(ctx: CommandContext, _args: SupportArguments) -> CommandResult {
    let disc_link = "https://www.discord.gg/h4BGGyR";
    let invite_link = "https://discord.com/oauth2/authorize?client_id=508968886998269962&scope=bot%20applications.commands&permissions=402738240";
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
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "Invite Link",
                format!(
                    "To invite the bot into your server: [Click Here]({})",
                    invite_link
                ),
            ),
        )
        .field(
            EmbedFieldBuilder::new(
                "Website",
                format!("To check out our website: [Click Here]({})", website),
            ),
        )
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;
    Ok(())
}
