mod new;
mod types;
mod view;

use crate::framework::prelude::*;

use new::*;
use types::*;
use view::*;

pub static EVENTS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["events", "event"],
    desc: Some("Command to view statistics about the events module of the server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &EVENT_TYPE_COMMAND,
        &EVENT_NEW_COMMAND,
        &EVENT_ATTENDEE_COMMAND,
    ],
    group: Some("Premium"),
};

pub static EVENTS_COMMAND: Command = Command {
    fun: events,
    options: &EVENTS_OPTIONS,
};

#[command]
pub async fn events(_ctx: &Context, _msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    Ok(())
}
