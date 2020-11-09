use crate::framework::prelude::*;

pub static EVENT_TYPE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["types", "type"],
    desc: Some("Command to view the created event types"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[&EVENT_TYPE_NEW_COMMAND, &EVENT_TYPE_MODIFY_COMMAND],
    group: None,
};

pub static EVENT_TYPE_COMMAND: Command = Command {
    fun: event_type,
    options: &EVENT_TYPE_OPTIONS,
};

pub static EVENT_TYPE_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to create a new event type"),
    usage: None,
    examples: &[],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_TYPE_NEW_COMMAND: Command = Command {
    fun: event_type_new,
    options: &EVENT_TYPE_NEW_OPTIONS,
};

pub static EVENT_TYPE_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["modify"],
    desc: Some("Command to modify an existing event type"),
    usage: None,
    examples: &[],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_TYPE_MODIFY_COMMAND: Command = Command {
    fun: event_type_modify,
    options: &EVENT_TYPE_MODIFY_OPTIONS,
};

#[command]
pub async fn event_type(_ctx: &Context, _msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    Ok(())
}

#[command]
pub async fn event_type_new(
    _ctx: &Context,
    _msg: &Message,
    _args: Arguments<'fut>,
) -> CommandResult {
    Ok(())
}

#[command]
pub async fn event_type_modify(
    _ctx: &Context,
    _msg: &Message,
    _args: Arguments<'fut>,
) -> CommandResult {
    Ok(())
}
