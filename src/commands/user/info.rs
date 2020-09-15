use crate::framework::prelude::*;
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_model::id::{UserId, GuildId};

pub static USERINFO_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["userinfo"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous")
};

pub static USERINFO_COMMAND: Command = Command {
    fun: userinfo,
    options: &USERINFO_OPTIONS
};

pub static BOTINFO_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["botinfo"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous")
};

pub static BOTINFO_COMMAND: Command = Command {
    fun: botinfo,
    options: &BOTINFO_OPTIONS
};

#[command]
pub async fn userinfo(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let author = match args.next().and_then(parse_role).and_then(|u| ctx.cache.user(UserId(u))) {
        Some(u) => (u.id, u.name.to_owned()),
        None => (msg.author.id, msg.author.name.to_owned())
    };
    let user = match ctx.database.get_user(author.0.0).await? {
        Some(u) => u,
        None => return Ok(())
    };

    let username = ctx.roblox.get_username_from_id(user.roblox_id).await?;

    let embed = EmbedBuilder::new()
        .title(author.1.clone()).unwrap()
        .description("Profile Information").unwrap()
        .field(EmbedFieldBuilder::new("Username", username).unwrap())
        .field(EmbedFieldBuilder::new("Roblox Id", user.roblox_id.to_string()).unwrap())
        .field(EmbedFieldBuilder::new("Discord Id", user.discord_id.to_string()).unwrap())
        //Put premium field here
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}

#[command]
pub async fn botinfo(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let current_user = ctx.cache.current_user().unwrap();
    let guilds = ctx.cache.guilds();
    let member_count: usize = guilds.iter().map(|g| ctx.cache.member_count(GuildId(*g))).sum();
    
    let embed = EmbedBuilder::new()
        .field(EmbedFieldBuilder::new("Name", format!("{}#{}", current_user.name, current_user.discriminator)).unwrap().inline())
        .field(EmbedFieldBuilder::new("Version", "2.5.0").unwrap().inline())
        .field(EmbedFieldBuilder::new("Language", "Rust").unwrap().inline())
        .field(EmbedFieldBuilder::new("Shards", ctx.cluster.config().shard_config().shard()[1].to_string()).unwrap().inline())
        .field(EmbedFieldBuilder::new("Shard Id", ctx.shard_id.to_string()).unwrap().inline())
        .field(EmbedFieldBuilder::new("Servers", guilds.len().to_string()).unwrap().inline())
        .field(EmbedFieldBuilder::new("Members", member_count.to_string()).unwrap().inline())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}