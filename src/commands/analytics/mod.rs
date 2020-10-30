mod register;
mod view;

use crate::framework::prelude::*;
use crate::models::guild::GuildType;
use itertools::Itertools;
use register::*;
use view::*;

pub static ANALYTICS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["analytics"],
    desc: Some("The analytics module"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &ANALYTICS_REGISTER_COMMAND,
        &ANALYTICS_UNREGISTER_COMMAND,
        &ANALYTICS_VIEW_COMMAND,
    ],
    group: Some("Administration"),
};

pub static ANALYTICS_COMMAND: Command = Command {
    fun: analytics,
    options: &ANALYTICS_OPTIONS,
};

#[command]
pub async fn analytics(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild = ctx
        .database
        .get_guild(msg.guild_id.unwrap().0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    if guild.settings.guild_type != GuildType::Beta {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Command Failed")
            .unwrap()
            .description("This module may only be used in Beta Tier Servers")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    if guild.registered_groups.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .unwrap()
            .title("Group Registration Failed")
            .unwrap()
            .description("There are no groups registered to this server")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let registered_groups = guild
        .registered_groups
        .iter()
        .map(|g| g.to_string())
        .join("\n");

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Registered Groups")
        .unwrap()
        .description(registered_groups)
        .unwrap()
        .build()
        .unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
