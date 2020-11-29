use crate::framework::prelude::*;

pub static TEST_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Creator,
    bucket: None,
    names: &["test"],
    desc: None,
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: true,
    sub_commands: &[],
    group: None,
};

pub static TEST_COMMAND: Command = Command {
    fun: test,
    options: &TEST_OPTIONS,
};

#[command]
pub async fn test(_ctx: &Context, _msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    Ok(())
}
