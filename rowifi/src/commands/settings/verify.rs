use rowifi_framework::prelude::*;
use rowifi_models::discord::id::RoleId;

#[derive(FromArgs)]
pub struct VerificationArguments {
    #[arg(help = "The Discord Role to set as the verification Role")]
    pub role: RoleId,
}

pub async fn settings_verification(
    ctx: CommandContext,
    args: VerificationArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get() as i64).await?;

    let verification_roles = vec![args.role.get() as i64];
    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verification_roles = $1 WHERE guild_id = $2",
            &[&verification_roles, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "The Verification Role was successfully set to <@&{}>",
            verification_roles[0]
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verification Role set to <@&{}>",
            verification_roles[0]
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct VerifiedArguments {
    #[arg(help = "The Discord Role to set as the verified Role")]
    pub role: RoleId,
}

pub async fn settings_verified(ctx: CommandContext, args: VerifiedArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get() as i64).await?;

    let verified_roles = vec![args.role.get() as i64];
    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verified_roles = $1 WHERE guild_id = $2",
            &[&verified_roles, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "The Verified Role was successfully set to <@&{}>",
            verified_roles[0]
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verified Role set to <@&{}>",
            verified_roles[0]
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
