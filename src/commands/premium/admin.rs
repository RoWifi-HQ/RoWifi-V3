use crate::framework::prelude::*;
use crate::models::user::{PremiumType, PremiumUser};

pub static PREMIUM_ADD_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Council,
    bucket: None,
    names: &["insert"],
    desc: None,
    usage: None,
    examples: &[],
    min_args: 2,
    hidden: true,
    sub_commands: &[],
    group: None,
};

pub static PREMIUM_DELETE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Creator,
    bucket: None,
    names: &["delete"],
    desc: None,
    usage: None,
    examples: &[],
    min_args: 1,
    hidden: true,
    sub_commands: &[],
    group: None,
};

pub static PREMIUM_CHECK_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Creator,
    bucket: None,
    names: &["check"],
    desc: None,
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: true,
    sub_commands: &[],
    group: None,
};

pub static PREMIUM_ADD_COMMAND: Command = Command {
    fun: premium_add,
    options: &PREMIUM_ADD_OPTIONS,
};

pub static PREMIUM_DELETE_COMMAND: Command = Command {
    fun: premium_delete,
    options: &PREMIUM_DELETE_OPTIONS,
};

pub static PREMIUM_CHECK_COMMAND: Command = Command {
    fun: premium_check,
    options: &PREMIUM_CHECK_OPTIONS,
};

#[command]
pub async fn premium_add(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    if !ctx.config.council.contains(&msg.author.id) {
        return Ok(());
    }

    let user_id: i64 = match args.next().map(str::parse) {
        Some(Ok(u)) => u,
        _ => return Ok(()),
    };

    let premium_type: i32 = match args.next().map(str::parse) {
        Some(Ok(p)) => p,
        _ => return Ok(()),
    };
    let premium_type: PremiumType = premium_type.into();
    //let existing_premium = ctx.database.get_premium(user_id as u64).await?.unwrap();
    // let mut servers = Vec::new();
    // for a in args {
    //     servers.push(a.parse::<i64>().unwrap());
    // }

    let premium_user = PremiumUser {
        discord_id: user_id,
        patreon_id: None,
        premium_type,
        discord_servers: Vec::new(),
        premium_owner: None,
        premium_patreon_owner: None,
    };
    ctx.database.add_premium(premium_user, false).await?;

    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .content(format!("Added Premium to {}", user_id))
        .unwrap()
        .await?;

    Ok(())
}

#[command]
pub async fn premium_delete(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let user_id: u64 = match args.next().map(str::parse) {
        Some(Ok(u)) => u,
        _ => return Ok(()),
    };

    ctx.database.delete_premium(user_id).await?;
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .content(format!("Successfully removed premium from {}", user_id))
        .unwrap()
        .await?;
    Ok(())
}

#[command]
pub async fn premium_check(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let premium_users = ctx.database.get_all_premium().await?;
    for user in premium_users {
        if user.premium_patreon_owner.is_some() {
            let (_patreon_id, tier) = ctx
                .patreon
                .get_patron(user.premium_owner.unwrap() as u64)
                .await?;
            if tier.is_none() {
                ctx.http
                    .create_message(msg.channel_id)
                    .content(format!(
                        "Transferred user {} with owner {} deleted from {:?}",
                        user.discord_id,
                        user.premium_owner.unwrap(),
                        user.premium_type
                    ))
                    .unwrap()
                    .await?;
                ctx.database.delete_premium(user.discord_id as u64).await?;
            }
        } else if user.patreon_id.is_some() {
            let (_patreon_id, tier) = ctx.patreon.get_patron(user.discord_id as u64).await?;
            if tier.is_none() {
                ctx.http
                    .create_message(msg.channel_id)
                    .content(format!(
                        "User {} deleted from {:?}",
                        user.discord_id, user.premium_type
                    ))
                    .unwrap()
                    .await?;
                ctx.database.delete_premium(user.discord_id as u64).await?;
            }
        }
    }
    Ok(())
}
