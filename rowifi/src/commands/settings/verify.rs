use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::id::RoleId;

#[derive(FromArgs)]
pub struct VerificationAddArguments {
    #[arg(help = "The Discord Roles to add as verification roles", rest)]
    pub roles: String,
}

pub async fn settings_verification_add(
    ctx: CommandContext,
    args: VerificationAddArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verification_roles = array_cat(verification_roles, $1::BIGINT[]) WHERE guild_id = $2",
            &[&role_ids, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "{} were added to the verification roles",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verification Roles: Added {}",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct VerificationRemoveArguments {
    #[arg(help = "The Discord Roles to remove as verification roles", rest)]
    pub roles: String,
}

pub async fn settings_verification_remove(
    ctx: CommandContext,
    args: VerificationRemoveArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;
    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let mut roles_to_keep = guild.verification_roles;
    roles_to_keep.retain(|r| !role_ids.contains(r));

    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verification_roles = $1 WHERE guild_id = $2",
            &[&roles_to_keep, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "{} were removed from the verification roles",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verification Roles: Removed {}",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct VerifiedAddArguments {
    #[arg(help = "The Discord Roles to add as verified roles", rest)]
    pub roles: String,
}

pub async fn settings_verified_add(
    ctx: CommandContext,
    args: VerifiedAddArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verified_roles = array_cat(verified_roles, $1::BIGINT[]) WHERE guild_id = $2",
            &[&role_ids, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "{} were added to the verified roles",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verified Roles: Added {}",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}

#[derive(FromArgs)]
pub struct VerifiedRemoveArguments {
    #[arg(help = "The Discord Roles to remove as verified roles", rest)]
    pub roles: String,
}

pub async fn settings_verified_remove(
    ctx: CommandContext,
    args: VerifiedRemoveArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let mut role_ids = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            role_ids.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    role_ids = role_ids.into_iter().unique().collect::<Vec<_>>();

    let mut roles_to_keep = guild.verified_roles;
    roles_to_keep.retain(|r| !role_ids.contains(r));

    ctx.bot
        .database
        .execute(
            "UPDATE guilds SET verified_roles = $1 WHERE guild_id = $2",
            &[&roles_to_keep, &guild.guild_id],
        )
        .await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(format!(
            "{} were removed from the verified roles",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Verified Roles: Removed {}",
            role_ids.iter().map(|r| format!("<@&{}>", r)).join(" ")
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
