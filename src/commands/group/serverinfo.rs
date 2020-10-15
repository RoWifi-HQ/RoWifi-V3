use crate::framework::prelude::*;
use twilight_embed_builder::EmbedFieldBuilder;

pub static SERVERINFO_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["serverinfo"],
    desc: Some("Shows the information about the server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Miscellanous"),
};

pub static SERVERINFO_COMMAND: Command = Command {
    fun: serverinfo,
    options: &SERVERINFO_OPTIONS,
};

#[command]
pub async fn serverinfo(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let embed = EmbedBuilder::new()
        .default_data()
        .field(
            EmbedFieldBuilder::new("Guild Id", guild_id.0.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Member Count", ctx.cache.member_count(guild_id).to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Shard Id", ctx.shard_id.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Tier", guild.settings.guild_type.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Prefix",
                guild.command_prefix.clone().unwrap_or_else(|| "!".into()),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Verification Role",
                format!("<@&{}>", guild.verification_role),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Verified Role", format!("<@&{}>", guild.verified_role))
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Rankbinds", guild.rankbinds.len().to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Groupbinds", guild.groupbinds.len().to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Custombinds", guild.custombinds.len().to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Assetbinds", guild.assetbinds.len().to_string())
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
