mod new;
mod types;
mod view;

use crate::framework::prelude::*;
use crate::models::guild::GuildType;

use new::EVENT_NEW_COMMAND;
use types::EVENT_TYPE_COMMAND;
use view::{EVENT_ATTENDEE_COMMAND, EVENT_HOST_COMMAND, EVENT_VIEW_COMMAND};

pub static EVENTS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Normal,
    bucket: None,
    names: &["events", "event"],
    desc: Some("Command to view information about the events module of the server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &EVENT_TYPE_COMMAND,
        &EVENT_NEW_COMMAND,
        &EVENT_ATTENDEE_COMMAND,
        &EVENT_HOST_COMMAND,
        &EVENT_VIEW_COMMAND,
    ],
    group: Some("Premium"),
};

pub static EVENTS_COMMAND: Command = Command {
    fun: events,
    options: &EVENTS_OPTIONS,
};

#[command]
pub async fn events(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
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
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let embed = EmbedBuilder::new().default_data()
        .title("Events Module").unwrap()
        .description("An amazing module of RoWifi to allow your members to log events they host and for you to track them").unwrap()
        .field(EmbedFieldBuilder::new("Event Types", "To register a new event type: `!event type new <Event Id> <Event Name>`\nTo modify an existing event type: `!event type modify <Event Id> <Event Name>`").unwrap())
        .field(EmbedFieldBuilder::new("For Trainers", "To add a new event: `!event new`").unwrap())
        .field(EmbedFieldBuilder::new("Viewing Events", "To see the last 12 events attended by the member: `!event attendee [RobloxName]`\nTo see the last 12 events hosted by the member: `!event host [RobloxName]`\nTo view specific information about an event: `!event view <Event Id>`").unwrap())
        .build().unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

    Ok(())
}
