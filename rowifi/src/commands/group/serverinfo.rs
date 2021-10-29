use rowifi_framework::prelude::*;

pub async fn serverinfo(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .field(EmbedFieldBuilder::new("Guild Id", guild_id.0.to_string()).inline())
        .field(
            EmbedFieldBuilder::new(
                "Member Count",
                ctx.bot.cache.member_count(guild_id).to_string(),
            )
            .inline(),
        )
        .field(EmbedFieldBuilder::new("Cluster Id", ctx.bot.cluster_id.to_string()).inline())
        .field(EmbedFieldBuilder::new("Tier", guild.settings.guild_type.to_string()).inline())
        .field(
            EmbedFieldBuilder::new(
                "Prefix",
                guild.command_prefix.clone().unwrap_or_else(|| "!".into()),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Verification Role",
                if let Some(verification_role) = guild.verification_role {
                    format!("<@&{}>", verification_role)
                } else {
                    "None".into()
                },
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Verified Role",
                if let Some(verified_role) = guild.verified_role {
                    format!("<@&{}>", verified_role)
                } else {
                    "None".into()
                },
            )
            .inline(),
        )
        .field(EmbedFieldBuilder::new("Rankbinds", guild.rankbinds.len().to_string()).inline())
        .field(EmbedFieldBuilder::new("Groupbinds", guild.groupbinds.len().to_string()).inline())
        .field(EmbedFieldBuilder::new("Custombinds", guild.custombinds.len().to_string()).inline())
        .field(EmbedFieldBuilder::new("Assetbinds", guild.assetbinds.len().to_string()).inline())
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed]).exec().await?;
    Ok(())
}
