use itertools::Itertools;
use rowifi_database::decrypt_bytes;
use rowifi_framework::prelude::*;
use rowifi_models::{
    discord::datetime::Timestamp,
    events::{EventLog, EventType},
    guild::GuildType,
    roblox::id::UserId as RobloxUserId,
    id::UserId
};

#[derive(FromArgs)]
pub struct EventAttendeeArguments {
    #[arg(help = "The roblox username of the attendee")]
    pub username: Option<String>,
}

pub async fn event_attendee(ctx: CommandContext, args: EventAttendeeArguments) -> CommandResult {
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

    let roblox_id = match args.username {
        Some(s) => match ctx.bot.roblox.get_user_from_username(&s).await? {
            Some(i) => i.id.0 as i64,
            None => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Event Viewing Failed")
                    .description("Given roblox username does not have an associated id")
                    .color(Color::Red as u32)
                    .build()
                    .unwrap();
                ctx.respond().embeds(&[embed])?.exec().await?;
                return Ok(());
            }
        },
        None => {
            let user = ctx
                .bot
                .database
                .get_linked_user(UserId(ctx.author.id), guild_id)
                .await?;
            match user {
                Some(u) => u.roblox_id,
                None => {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .title("Event Viewing Failed")
                        .description("You must be verified to use this command on yourself")
                        .color(Color::Red as u32)
                        .build()
                        .unwrap();
                    ctx.respond().embeds(&[embed])?.exec().await?;
                    return Ok(());
                }
            }
        }
    };
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
        .query::<EventLog>(
            "SELECT * FROM events WHERE guild_id = $1 AND $2 = ANY(attendees)",
            &[&(guild_id), &roblox_id],
        )
        .await?;

    let mut pages = Vec::new();
    let mut page_count = 0;

    for events in events.chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Attended Events")
            .description(format!("Page {}", page_count + 1));

        for event in events {
            let name = format!("Id: {}", event.guild_event_id);

            let event_type = event_types
                .iter()
                .find(|e| e.event_type_guild_id == event.event_type)
                .unwrap();
            let host = ctx
                .bot
                .roblox
                .get_user(RobloxUserId(event.host_id as u64), false)
                .await?;
            let desc = format!(
                "Event Type: {}\nHost: {}\nTimestamp: <t:{}:f>",
                event_type.name,
                host.name,
                event.timestamp.timestamp()
            );

            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline());
        }
        pages.push(embed.build()?);
        page_count += 1;
    }

    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct EventHostArguments {
    #[arg(help = "The Roblox Username of the host")]
    pub username: Option<String>,
}

pub async fn event_host(ctx: CommandContext, args: EventHostArguments) -> CommandResult {
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

    let roblox_id = match args.username {
        Some(s) => match ctx.bot.roblox.get_user_from_username(&s).await? {
            Some(i) => i.id.0 as i64,
            None => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Event Viewing Failed")
                    .description("Given roblox username does not have an associated id")
                    .color(Color::Red as u32)
                    .build()
                    .unwrap();
                ctx.respond().embeds(&[embed])?.exec().await?;
                return Ok(());
            }
        },
        None => {
            let user = ctx
                .bot
                .database
                .get_linked_user(UserId(ctx.author.id), guild_id)
                .await?;
            match user {
                Some(u) => u.roblox_id,
                None => {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .title("Event Viewing Failed")
                        .description("You must be verified to use this command on yourselves")
                        .color(Color::Red as u32)
                        .build()
                        .unwrap();
                    ctx.respond().embeds(&[embed])?.exec().await?;
                    return Ok(());
                }
            }
        }
    };

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
        .query::<EventLog>(
            "SELECT * FROM events WHERE guild_id = $1 AND host_id = $2",
            &[&(guild_id), &roblox_id],
        )
        .await?;

    let mut pages = Vec::new();
    let mut page_count = 0;

    for events in events.chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Hosted Events")
            .description(format!("Page {}", page_count + 1));

        for event in events {
            let name = format!("Id: {}", event.guild_event_id);

            let event_type = event_types
                .iter()
                .find(|e| e.event_type_guild_id == event.event_type)
                .unwrap();
            let host = ctx
                .bot
                .roblox
                .get_user(RobloxUserId(event.host_id as u64), false)
                .await?;
            let desc = format!(
                "Event Type: {}\nHost: {}\nTimestamp: <t:{}:f>",
                event_type.name,
                host.name,
                event.timestamp.timestamp()
            );

            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline());
        }
        pages.push(embed.build()?);
        page_count += 1;
    }

    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct EventViewArguments {
    #[arg(help = "The ID of the event to be viewed")]
    pub event_id: i64,
}

pub async fn event_view(ctx: CommandContext, args: EventViewArguments) -> CommandResult {
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

    let event_id = args.event_id;
    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id)],
        )
        .await?;
    let event = ctx
        .bot
        .database
        .query_opt::<EventLog>(
            "SELECT * FROM events WHERE guild_id = $1 AND guild_event_id = $2",
            &[&(guild_id), &event_id],
        )
        .await?;

    let event = match event {
        Some(e) => e,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Event Viewing Failed")
                .description(format!("An event with id {} does not exist", event_id))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let event_type = event_types
        .iter()
        .find(|e| e.event_type_guild_id == event.event_type)
        .unwrap();
    let host = ctx
        .bot
        .roblox
        .get_user(RobloxUserId(event.host_id as u64), false)
        .await?;
    let mut attendees = Vec::new();
    for a in &event.attendees {
        let roblox_name = ctx
            .bot
            .roblox
            .get_user(RobloxUserId(*a as u64), false)
            .await?;
        attendees.push(roblox_name);
    }

    let mut embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Event Id: {}", event.guild_event_id))
        .field(EmbedFieldBuilder::new(
            "Event Type",
            event_type.name.clone(),
        ))
        .field(EmbedFieldBuilder::new("Host", host.name))
        .timestamp(Timestamp::from_secs(event.timestamp.timestamp()).unwrap());

    if !event.attendees.is_empty() {
        embed = embed.field(EmbedFieldBuilder::new(
            "Attendees",
            attendees.iter().map(|a| format!("- {}", a.name)).join("\n"),
        ));
    }

    if let Some(notes_bytes) = &event.notes {
        let notes = decrypt_bytes(
            notes_bytes,
            &ctx.bot.database.cipher,
            guild_id.get(),
            event.host_id as u64,
            event.timestamp.timestamp() as u64,
        );
        embed = embed.field(EmbedFieldBuilder::new(
            "Notes",
            String::from_utf8(notes).unwrap(),
        ));
    }

    ctx.respond().embeds(&[embed.build()?])?.exec().await?;
    Ok(())
}
