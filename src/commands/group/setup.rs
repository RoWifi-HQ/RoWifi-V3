use crate::framework::prelude::*;
use crate::models::guild::RoGuild;
use tokio::time::timeout;
use std::time::Duration;
use twilight_model::gateway::payload::MessageCreate;

pub static SETUP_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["setup"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: Some("Administration")
};

pub static SETUP_COMMAND: Command = Command {
    fun: setup,
    options: &SETUP_OPTIONS
};

#[command]
pub async fn setup(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let existing_guild = ctx.database.get_guild(msg.guild_id.unwrap().0).await?;

    ctx.http.create_message(msg.channel_id).content("Which role would you like to bind as your **unverified role**?\nPlease tag the role for the bot to be able to detect it.").unwrap().await?;
    let id = msg.author.id;
    let verification_fut = ctx.standby.wait_for_message(msg.channel_id, 
        move |event: &MessageCreate| event.author.id == id && !event.content.is_empty() && parse_role(event.content.to_owned()).is_some());
    let verification_role = match timeout(Duration::from_secs(300), verification_fut).await {
        Ok(Ok(m)) => parse_role(m.content.to_owned()).unwrap(),
        _ => {
            //Do this later
            return Ok(())
        }
    };

    ctx.http.create_message(msg.channel_id).content("Which role would you like to bind as your **verifed role**?\n Please tag the role for the bot to be able to detect it.").unwrap().await?;
    let verified_fut = ctx.standby.wait_for_message(msg.channel_id, 
        move |event: &MessageCreate| event.author.id == id && !event.content.is_empty() && parse_role(event.content.to_owned()).is_some());
    let verified_role = match timeout(Duration::from_secs(300), verified_fut).await {
        Ok(Ok(m)) => parse_role(m.content.to_owned()).unwrap(),
        _ => {
            return Ok(())
        }
    };

    let mut replace = false;
    let mut guild = RoGuild::default();
    guild.id = msg.guild_id.unwrap().0 as i64;
    guild.verification_role = verification_role as i64;
    guild.verified_role = verified_role as i64;
    if let Some(existing) = existing_guild {
        guild.command_prefix = existing.command_prefix;
        replace = true;
    }
    
    ctx.database.add_guild(guild, replace).await?;
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Setup Successful!").unwrap()
        .description("Server has been setup successfully. Use `rankbinds new` or `groupbinds new` to start setting up your binds").unwrap()
        .build().unwrap();
    ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    Ok(())
}