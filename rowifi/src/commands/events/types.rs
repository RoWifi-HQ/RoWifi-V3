use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::{events::EventType, guild::GuildType};

use super::EventArguments;

pub async fn event_type(ctx: CommandContext, _args: EventArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommandError::NoRoGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Command Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let mut embed = EmbedBuilder::new()
        .default_data()
        .title("Event Types")
        .unwrap();
    for event_type in &guild.event_types {
        let name = format!("Id: {}", event_type.id);
        let value = format!("Name: {}", event_type.name);
        embed = embed.field(EmbedFieldBuilder::new(name, value).unwrap().inline());
    }
    let embed = embed.build().unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

#[derive(FromArgs)]
pub struct EventTypeArguments {
    #[arg(help = "The Event Id to assign/modify")]
    pub event_id: i64,
    #[arg(help = "The Event Name to assign/modify", rest)]
    pub event_name: String,
}

pub async fn event_type_new(ctx: CommandContext, args: EventTypeArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommandError::NoRoGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Command Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let event_id = args.event_id;
    let event_name = args.event_name;

    if guild.event_types.iter().any(|e| e.id == event_id) {
        let embed = EmbedBuilder::new().default_data().color(Color::Red as u32).unwrap()
            .title("Event Type Addition Failed").unwrap()
            .description(format!("An event type with id {} already exists. To modify an event type, use `events type modify`", event_id)).unwrap()
            .build().unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let event_type = EventType {
        id: event_id,
        name: event_name.to_string(),
        xp: 0,
    };
    let event_bson = to_bson(&event_type)?;

    let filter = doc! {"_id": guild_id.0};
    let update = doc! {"$push": {"EventTypes": event_bson}};

    ctx.bot.database.modify_guild(filter, update).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Event Type Addition Successful")
        .unwrap()
        .field(
            EmbedFieldBuilder::new(
                format!("Id: {}", event_type.id),
                format!("Name: {}", event_type.name),
            )
            .unwrap(),
        )
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}

pub async fn event_type_modify(ctx: CommandContext, args: EventTypeArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommandError::NoRoGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Command Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let event_id = args.event_id;
    let event_name = args.event_name;
    let event_type_index = match guild.event_types.iter().position(|e| e.id == event_id) {
        Some(i) => i,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Event Type Modification Failed")
                .unwrap()
                .description(format!("An event type with id {} does not exist", event_id))
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    let event = &guild.event_types[event_type_index];

    let filter = doc! {"_id": guild_id.0};
    let index_str = format!("EventTypes.{}.Name", event_type_index);
    let update = doc! {"$set": {index_str: event_name.clone()}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Event Type Id: {}", event.id);
    let desc = format!("Name: {} -> {}", event.name.clone(), event_name);
    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .title("Event Type Modification Successful")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, desc).unwrap())
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
