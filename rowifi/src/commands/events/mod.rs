mod new;
mod reset;
mod summary;
mod types;
mod view;

use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

use new::events_new;
use reset::event_reset;
use summary::event_summary;
use types::{event_type, event_type_disable, event_type_enable, event_type_modify, event_type_new};
use view::{event_attendee, event_host, event_view};

pub fn events_config(cmds: &mut Vec<Command>) {
    let event_types_new_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to add a new event type")
        .handler(event_type_new);

    let event_types_modify_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify an existing event type")
        .handler(event_type_modify);

    let event_types_disable_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["disable"])
        .description("Command to disable an event type. This prevents trainers from logging new events with this type")
        .handler(event_type_disable);

    let event_types_enable_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["enable"])
        .description("Command to enable an event type for logging")
        .handler(event_type_enable);

    let event_types_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view the event types")
        .handler(event_type);

    let event_types_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["types", "type"])
        .description("Command to view the event types")
        .sub_command(event_types_new_cmd)
        .sub_command(event_types_modify_cmd)
        .sub_command(event_types_disable_cmd)
        .sub_command(event_types_enable_cmd)
        .sub_command(event_types_view_cmd)
        .handler(event_type);

    let events_new_cmd = Command::builder()
        .level(RoLevel::Trainer)
        .names(&["new"])
        .description("Command for users with `RoWifi Trainer` to log an event")
        .handler(events_new);

    let events_attendee_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["attendee"])
        .description("Command to view the last 12 events of an user")
        .handler(event_attendee);

    let events_host_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["host"])
        .description("Command to view the last 12 events hosted by an user")
        .handler(event_host);

    let events_view_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["view"])
        .description("Command to view information about a specific event")
        .handler(event_view);

    let events_reset_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["reset"])
        .description("Command to reset the events subsystem")
        .handler(event_reset);

    let events_summary_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["summary"])
        .description("Command to view the summary of all logged events")
        .handler(event_summary);

    let events_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["event", "events"])
        .description("Module to interact with the events subsystem")
        .group("Premium")
        .sub_command(events_new_cmd)
        .sub_command(event_types_cmd)
        .sub_command(events_attendee_cmd)
        .sub_command(events_host_cmd)
        .sub_command(events_view_cmd)
        .sub_command(events_reset_cmd)
        .sub_command(events_summary_cmd)
        .handler(events);
    cmds.push(events_cmd);
}

pub async fn events(ctx: CommandContext) -> CommandResult {
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

    let embed = EmbedBuilder::new().default_data()
        .title("Events Module")
        .description("An amazing module of RoWifi to allow your members to log events they host and for you to track them")
        .field(EmbedFieldBuilder::new("Event Types", "To register a new event type: `!event type new <Event Id> <Event Name>`\nTo modify an existing event type: `!event type modify <Event Id> <Event Name>`"))
        .field(EmbedFieldBuilder::new("For Trainers", "To add a new event: `!event new`"))
        .field(EmbedFieldBuilder::new("Viewing Events", "To see the last 12 events attended by the member: `!event attendee [RobloxName]`\nTo see the last 12 events hosted by the member: `!event host [RobloxName]`\nTo view specific information about an event: `!event view <Event Id>`"))
        .build()?;
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
