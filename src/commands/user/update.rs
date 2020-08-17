use crate::framework::prelude::*;
use twilight::model::id::UserId;
use std::sync::Arc;

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
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    let server = ctx.cache.guild(guild_id).await.unwrap();

    let user_id = match args.next() {
        Some(s) => match parse_username(s).await {
            Some(id) => UserId(id),
            None => msg.author.id
        },
        None => msg.author.id
    };

    let member = match ctx.member(guild_id, user_id).await? {
        Some(m) => m,
        None => {
            //Error
            println!("No member found");
            return Ok(())
        }
    };

    //Check for server owner
    // if server.owner_id.0 == member.user.id.0 {
    //     //Send Embed
    //     return Ok(())
    // }

    //Handle role position check

    //Check for bypass role
    let bypass = ctx.cache.bypass_roles(guild_id).await; 
    if let Some(bypass_role) = &bypass.0 {
        if member.roles.contains(bypass_role) {
            //Send Embed
            return Ok(())
        }
    }

    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => {
            //Send message about being unverified
            return Ok(())
        }
    };

    let guild = match ctx.database.get_guild(&guild_id.0).await? {
        Some(g) => g,
        None => {
            //Send message about unsetup guild
            return Ok(())
        }
    }; 
    let guild_roles = ctx.cache.roles(guild_id).await;

    let _ = user.update(Arc::clone(&ctx.http), member, Arc::clone(&ctx.roblox), server, &guild, guild_roles).await;
    let end = chrono::Utc::now();
    let _ = ctx.http.create_message(msg.channel_id).content(format!("Execution Time: {:?}", end - start)).unwrap().await;

    Ok(())
}