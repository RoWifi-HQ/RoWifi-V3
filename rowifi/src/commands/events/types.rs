use rowifi_framework::prelude::*;
use rowifi_models::{events::EventType, guild::GuildType};

pub async fn event_type(ctx: CommandContext) -> CommandResult {
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

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    let mut embed = EmbedBuilder::new().default_data().title("Event Types");
    for event_type in &event_types {
        let name = format!("Id: {}", event_type.event_type_guild_id);
        let value = format!("Name: {}", event_type.name);
        embed = embed.field(EmbedFieldBuilder::new(name, value).inline());
    }
    ctx.respond().embeds(&[embed.build()?])?.exec().await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct EventTypeArguments {
    #[arg(help = "The Event Id to assign/modify")]
    pub event_id: i32,
    #[arg(help = "The Event Name to assign/modify", rest)]
    pub event_name: String,
}

pub async fn event_type_new(ctx: CommandContext, args: EventTypeArguments) -> CommandResult {
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

    let event_id = args.event_id;
    let event_name = args.event_name;

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    if event_types
        .iter()
        .any(|e| e.event_type_guild_id == event_id)
    {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32)
            .title("Event Type Addition Failed")
            .description(format!("An event type with id {} already exists. To modify an event type, use `events type modify`", event_id))
            .build()?;
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let event_type = EventType {
        event_type_id: 0,
        event_type_guild_id: event_id,
        guild_id: guild_id.get() as i64,
        name: event_name.to_string(),
        disabled: false,
    };

    ctx.bot.database.execute("INSERT INTO event_types(event_type_guild_id, guild_id, name, disabled) VALUES($1, $2, $3, $4)", &[&event_type.event_type_guild_id, &event_type.guild_id, &event_type.name, &event_type.disabled]).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Event Type Addition Successful")
        .field(EmbedFieldBuilder::new(
            format!("Id: {}", event_type.event_type_guild_id),
            format!("Name: {}", event_type.name),
        ))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

pub async fn event_type_modify(ctx: CommandContext, args: EventTypeArguments) -> CommandResult {
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

    let event_type_guild_id = args.event_id;
    let event_name = args.event_name;

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;
    let event = match event_types
        .iter()
        .find(|e| e.event_type_guild_id == event_type_guild_id)
    {
        Some(e) => e,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Event Type Modification Failed")
                .description(format!(
                    "An event type with id {} does not exist",
                    event_type_guild_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    ctx.bot
        .database
        .execute(
            "UPDATE event_types SET name = $1 WHERE event_type_id = $2",
            &[&event_name, &event.event_type_id],
        )
        .await?;

    let name = format!("Event Type Id: {}", event.event_type_id);
    let desc = format!("Name: {} -> {}", &event.name, event_name);
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Event Type Modification Successful")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct DisableArguments {
    #[arg(help = "The event id to disable")]
    pub event_id: i32,
}

pub async fn event_type_disable(ctx: CommandContext, args: DisableArguments) -> CommandResult {
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

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    let event_type_guild_id = args.event_id;
    let event = match event_types
        .iter()
        .find(|e| e.event_type_guild_id == event_type_guild_id)
    {
        Some(i) => i,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Event Type Modification Failed")
                .description(format!(
                    "An event type with id {} does not exist",
                    event_type_guild_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    ctx.bot
        .database
        .execute(
            "UPDATE event_types SET disabled = $1 WHERE event_type_id = $2",
            &[&true, &event.event_type_id],
        )
        .await?;

    let name = format!("Event Type Id: {}", event.event_type_guild_id);
    let desc = format!("Disabled: {} -> {}", event.disabled, true);
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Event Type Modification Successful")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct EnableArguments {
    #[arg(help = "The event id to enable")]
    pub event_id: i32,
}

pub async fn event_type_enable(ctx: CommandContext, args: EnableArguments) -> CommandResult {
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

    let event_types = ctx
        .bot
        .database
        .query::<EventType>(
            "SELECT * FROM event_types WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    let event_type_guild_id = args.event_id;
    let event = match event_types
        .iter()
        .find(|e| e.event_type_guild_id == event_type_guild_id)
    {
        Some(e) => e,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Event Type Modification Failed")
                .description(format!(
                    "An event type with id {} does not exist",
                    event_type_guild_id
                ))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    ctx.bot
        .database
        .execute(
            "UPDATE event_types SET disabled = $1 WHERE event_type_id = $2",
            &[&false, &event.event_type_id],
        )
        .await?;

    let name = format!("Event Type Id: {}", event.event_type_guild_id);
    let desc = format!("Disabled: {} -> {}", event.disabled, false);
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Event Type Modification Successful")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}
