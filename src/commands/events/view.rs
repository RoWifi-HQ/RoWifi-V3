use crate::framework::prelude::*;

pub static EVENT_ATTENDEE_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["attendee"],
    desc: Some("Command to view the last 12 events attended by the given user"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static EVENT_ATTENDEE_COMMAND: Command = Command {
    fun: event_attendee,
    options: &EVENT_ATTENDEE_OPTIONS,
};

#[command]
pub async fn event_attendee(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let user_id = match args.next().and_then(|x| x.parse::<i64>().ok()) {
        Some(s) => s,
        None => {
            let user = ctx.database.get_user(msg.author.id.0).await?;
            match user {
                Some(u) => u.roblox_id as i64,
                None => {
                    //Give unverified error
                    return Ok(());
                }
            }
        }
    };

    ctx.database.get_events(guild_id.0 as i64, user_id).await?;
    Ok(())
}
