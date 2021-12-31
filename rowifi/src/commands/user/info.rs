use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::id::{UserId},
    roblox::id::UserId as RobloxUserId,
    id::GuildId,
};

#[derive(FromArgs)]
pub struct UserInfoArguments {
    pub user: Option<UserId>,
}

pub async fn userinfo(ctx: CommandContext, args: UserInfoArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let author = if let Some(user) = args.user {
        let u = ctx.member(guild_id, user).await?;
        if let Some(u) = u {
            (u.user.id, u.user.name.clone())
        } else {
            let author_user = ctx.member(guild_id, ctx.author.id).await?;
            if let Some(a) = author_user {
                (a.user.id, a.user.name.clone())
            } else {
                return Ok(());
            }
        }
    } else {
        let author_user = ctx.member(guild_id, ctx.author.id).await?;
        if let Some(a) = author_user {
            (a.user.id, a.user.name.clone())
        } else {
            return Ok(());
        }
    };

    let user = match ctx
        .bot
        .database
        .get_linked_user(author.0.get() as i64, ctx.guild_id.unwrap())
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("User Info Failed")
                .description("User was not verified. Please ask him/her to verify themselves")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let roblox_user = ctx
        .bot
        .roblox
        .get_user(RobloxUserId(user.roblox_id as u64), false)
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title(author.1.clone())
        .description("Profile Information")
        .field(EmbedFieldBuilder::new("Username", roblox_user.name.clone()))
        .field(EmbedFieldBuilder::new(
            "Roblox Id",
            user.roblox_id.to_string(),
        ))
        .field(EmbedFieldBuilder::new(
            "Discord Id",
            user.discord_id.to_string(),
        ))
        .thumbnail(
            ImageSource::url(format!(
                "https://www.roblox.com/Thumbs/Avatar.ashx?x=150&y=150&Format=Png&username={}",
                roblox_user.name
            ))
            .unwrap(),
        )
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

pub async fn botinfo(ctx: CommandContext) -> CommandResult {
    let current_user = ctx.bot.cache.current_user().unwrap();
    let guilds = ctx.bot.cache.guilds();
    let member_count: i64 = guilds
        .iter()
        .map(|g| ctx.bot.cache.member_count(GuildId::new(*g)))
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
        .field(EmbedFieldBuilder::new("Version", env!("CARGO_PKG_VERSION")).inline())
        .field(EmbedFieldBuilder::new("Language", "Rust").inline())
        .field(EmbedFieldBuilder::new("Shards", ctx.bot.total_shards.to_string()).inline())
        .field(EmbedFieldBuilder::new("Cluster Id", ctx.bot.cluster_id.to_string()).inline())
        .field(EmbedFieldBuilder::new("Servers", guilds.len().to_string()).inline())
        .field(EmbedFieldBuilder::new("Members", member_count.to_string()).inline())
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

pub async fn support(ctx: CommandContext) -> CommandResult {
    let disc_link = "https://www.discord.gg/h4BGGyR";
    let invite_link = "https://discord.com/oauth2/authorize?client_id=508968886998269962&scope=bot%20applications.commands&permissions=402738240";
    let website = "https://rowifi.link";
    let embed = EmbedBuilder::new()
        .default_data()
        .field(EmbedFieldBuilder::new(
            "Support Server",
            format!(
                "To know more about announcements, updates and other stuff: [Click Here]({})",
                disc_link
            ),
        ))
        .field(EmbedFieldBuilder::new(
            "Invite Link",
            format!(
                "To invite the bot into your server: [Click Here]({})",
                invite_link
            ),
        ))
        .field(EmbedFieldBuilder::new(
            "Website",
            format!("To check out our website: [Click Here]({})", website),
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}
