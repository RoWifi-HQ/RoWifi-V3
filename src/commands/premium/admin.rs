use crate::framework::prelude::*;
use crate::models::user::{PremiumType, PremiumUser};

pub static PREMIUM_ADD_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Creator,
    bucket: None,
    names: &["add"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: true,
    sub_commands: &[],
    group: None
};

pub static PREMIUM_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Creator,
    bucket: None,
    names: &["delete"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    min_args: 0,
    hidden: true,
    sub_commands: &[],
    group: None
};

pub static PREMIUM_ADD_COMMAND: Command = Command {
    fun: premium_add,
    options: &PREMIUM_ADD_OPTIONS
};

pub static PREMIUM_DELETE_COMMAND: Command = Command {
    fun: premium_delete,
    options: &PREMIUM_DELETE_OPTIONS
};

#[command]
pub async fn premium_add(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let user_id = match args.next().map(|a| a.parse::<i64>()) {
        Some(Ok(u)) => u,
        _ => return Ok(())
    };

    let premium_type = match args.next().map(|a| a.parse::<i32>()) {
        Some(Ok(p)) => p,
        _ => return Ok(())
    };
    let premium_type: PremiumType = premium_type.into();

    let mut servers = Vec::new();
    for a in args {
        servers.push(a.parse::<i64>().unwrap());
    }

    let premium_user = PremiumUser {discord_id: user_id, patreon_id: None, premium_type, discord_servers: servers};
    ctx.database.add_premium(premium_user, false).await?;

    let _ = ctx.http.create_message(msg.channel_id).content(format!("Added Premium to {}", user_id)).unwrap().await?;

    Ok(())
}

#[command]
pub async fn premium_delete(_ctx: &Context, _msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let _user_id = match args.next().map(|a| a.parse::<i64>()) {
        Some(Ok(u)) => u,
        _ => return Ok(())
    };
    Ok(())
}