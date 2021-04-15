use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use twilight_model::id::RoleId;

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
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let verification_role = args.role.0;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"VerificationRole": verification_role}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(format!(
            "The Verification Role was successfully set to <@&{}>",
            verification_role
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description(format!(
            "Settings Modification: Verification Role set to <@&{}>",
            verification_role
        ))
        .unwrap()
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
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let verified_role = args.role.0;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"VerifiedRole": verified_role}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Settings Modification Successful")
        .unwrap()
        .description(format!(
            "The Verified Role was successfully set to <@&{}>",
            verified_role
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description(format!(
            "Settings Modification: Verified Role set to <@&{}>",
            verified_role
        ))
        .unwrap()
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
