use crate::framework::prelude::*;

pub static UPDATE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update", "getroles"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static UPDATE_COMMAND: Command = Command {
    fun: update,
    options: &UPDATE_OPTIONS
};

#[command]
pub async fn update(ctx: &Context, msg: &Message, _args: Arguments<'_>) -> CommandResult {
    let _ = ctx.http.create_message(msg.channel_id).content("Update Works!").unwrap().await;
    Ok(())
}