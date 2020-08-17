use crate::framework::prelude::*;

pub static VERIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["verify"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static VERIFY_COMMAND: Command = Command {
    fun: verify,
    options: &VERIFY_OPTIONS
};

#[command]
pub async fn verify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult { 
    Ok(())
}