use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

pub async fn event_reset(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get() as i64).await?;

    if guild.kind != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let confirmation = await_confirmation(
        "Are you sure you would like to delete all logged events & reset all event types?",
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
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let event_types_change = transaction.prepare_cached("DELETE FROM event_types WHERE guild_id = $1").await?;
    transaction.execute(&event_types_change, &[&(guild_id.get() as i64)]).await?;

    let events_change = transaction.prepare_cached("DELETE FROM events WHERE guild_id = $1").await?;
    transaction.execute(&events_change, &[&(guild_id.get() as i64)]).await?;

    transaction.commit().await?;

    ctx.respond()
        .content("The event system has been reset successfully")?
        .exec()
        .await?;

    Ok(())
}
