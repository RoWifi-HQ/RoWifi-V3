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
pub async fn update(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let start = chrono::Utc::now();
    let server = ctx.cache.guild(msg.guild_id.unwrap()).await?.unwrap();

    let member = match args.next() {
        Some(s) => match ctx.parse_member(server.id, s).await? {
            Some(m) => m,
            None => {
                //Give error
                return Ok(())
            }
        },
        None => ctx.get_member(server.id, msg.author.id).await.unwrap()
    };

    if server.owner_id.0 == member.user.id.0 {
        //Give error
        return Ok(())
    }

    //TODO: Role position check

    //let bypass = ctx.cache.role(role_id).await

    Ok(())
}