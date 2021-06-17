mod new;
mod types;
mod view;

use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

use new::events_new;
use types::{event_type, event_type_modify, event_type_new};
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

    let event_types_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["types", "type"])
        .description("Command to view the event types")
        .sub_command(event_types_new_cmd)
        .sub_command(event_types_modify_cmd)
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
        .handler(events);
    cmds.push(events_cmd);
}

#[derive(FromArgs)]
pub struct EventArguments {}

pub async fn events(ctx: CommandContext, _args: EventArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This module may only be used in Beta Tier Servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let embed = EmbedBuilder::new().default_data()
        .title("Events Module")
        .description("An amazing module of RoWifi to allow your members to log events they host and for you to track them")
        .field(EmbedFieldBuilder::new("Event Types", "To register a new event type: `!event type new <Event Id> <Event Name>`\nTo modify an existing event type: `!event type modify <Event Id> <Event Name>`"))
        .field(EmbedFieldBuilder::new("For Trainers", "To add a new event: `!event new`"))
        .field(EmbedFieldBuilder::new("Viewing Events", "To see the last 12 events attended by the member: `!event attendee [RobloxName]`\nTo see the last 12 events hosted by the member: `!event host [RobloxName]`\nTo view specific information about an event: `!event view <Event Id>`"))
        .build().unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
