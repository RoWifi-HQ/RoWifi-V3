use mongodb::bson::{doc, Document};
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

pub async fn event_reset(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
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
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"EventCounter": 0, "EventTypes": []}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let client = ctx.bot.database.as_ref();
    let events = client.database("Events").collection::<Document>("Logs");
    let filter = doc! {"GuildId": guild.id};
    let _res = events
        .delete_many(filter, None)
        .await
        .map_err(|d| RoError::Database(d.into()))?;

    ctx.respond()
        .content("The event system has been reset successfully")
        .exec()
        .await?;

    Ok(())
}
