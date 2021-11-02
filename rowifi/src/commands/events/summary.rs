use chrono::{Duration as CDuration, Utc};
use itertools::Itertools;
use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

pub async fn event_summary(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    if guild.settings.guild_type != GuildType::Beta {
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

    let pipeline = vec![doc! {"$match": {"GuildId": guild_id.0.get() as i64}}];
    let events = ctx.bot.database.get_events(pipeline).await?;

    let mut embed = EmbedBuilder::new().default_data().title("Events Summary");

    let event_groups = events
        .iter()
        .sorted_unstable_by_key(|e| e.event_type)
        .group_by(|e| e.event_type);
    for event_group in &event_groups {
        let event_name = guild
            .event_types
            .iter()
            .find(|e| e.id == event_group.0)
            .unwrap();

        let all_events = event_group.1.collect::<Vec<_>>();
        let total = all_events.len();
        let last_30_days = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp.to_chrono()) <= CDuration::days(30))
            .count();
        let last_7_days = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp.to_chrono()) <= CDuration::weeks(1))
            .count();
        let last_24_hours = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp.to_chrono()) <= CDuration::hours(24))
            .count();

        embed = embed.field(
            EmbedFieldBuilder::new(
                &event_name.name,
                format!(
                    "Total Events Hosted: {}\nHosted in last 30 days: {}\nHosted in 7 days: {}\nHosted in last 24 hours: {}",
                    total, last_30_days, last_7_days, last_24_hours
                )
            )
        );
    }

    ctx.respond()
        .embeds(&[embed.build().unwrap()])?
        .exec()
        .await?;

    Ok(())
}
