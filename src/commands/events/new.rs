use crate::framework::prelude::*;

pub static EVENT_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to view statistics about the events module of the server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_NEW_COMMAND: Command = Command {
    fun: event_new,
    options: &EVENT_NEW_OPTIONS,
};

#[command]
pub async fn event_new(_ctx: &Context, _msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    Ok(())
}
