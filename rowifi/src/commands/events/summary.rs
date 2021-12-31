use chrono::{Duration as CDuration, Utc};
use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::{
    events::{EventLog, EventType},
    guild::GuildType,
};

pub async fn event_summary(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

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

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id)],
        )
        .await?;
    let events = ctx
        .bot
        .database
        .query::<EventLog>("SELECT * FROM events WHERE guild_id = $1", &[&(guild_id)])
        .await?;

    let mut embed = EmbedBuilder::new().default_data().title("Events Summary");

    let event_groups = events
        .iter()
        .sorted_unstable_by_key(|e| e.event_type)
        .group_by(|e| e.event_type);
    for event_group in &event_groups {
        let event_name = event_types
            .iter()
            .find(|e| e.event_type_guild_id == event_group.0)
            .unwrap();

        let all_events = event_group.1.collect::<Vec<_>>();
        let total = all_events.len();
        let last_30_days = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp) <= CDuration::days(30))
            .count();
        let last_7_days = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp) <= CDuration::weeks(1))
            .count();
        let last_24_hours = all_events
            .iter()
            .filter(|e| (Utc::now() - e.timestamp) <= CDuration::hours(24))
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

    ctx.respond().embeds(&[embed.build()?])?.exec().await?;

    Ok(())
}
