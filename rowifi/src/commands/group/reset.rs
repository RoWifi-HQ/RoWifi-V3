use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

pub async fn reset(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

    let confirmation = await_confirmation(
        "Are you sure you would like to reset all binds & configurations?",
        &ctx,
    )
    .await?;
    if !confirmation {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Reset was cancelled!")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let guild = ctx.bot.database.get_guild(guild_id.0).await?;
    let guild = RoGuild {
        id: guild_id.0 as i64,
        command_prefix: guild.command_prefix,
        event_counter: guild.event_counter,
        ..RoGuild::default()
    };

    ctx.bot.database.add_guild(&guild, true).await?;

    ctx.bot.admin_roles.remove(&guild_id);
    ctx.bot.trainer_roles.remove(&guild_id);
    ctx.bot.bypass_roles.remove(&guild_id);
    ctx.bot.nickname_bypass_roles.remove(&guild_id);
    ctx.bot.log_channels.remove(&guild_id);

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Reset Successful!")
        .description("Your settings & bind configurations have been reset")
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
