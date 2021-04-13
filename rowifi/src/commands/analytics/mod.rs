mod register;
mod view;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;
use std::string::ToString;

use register::{analytics_register, analytics_unregister};
use view::analytics_view;

pub fn analytics_config(cmds: &mut Vec<Command>) {
    let analytics_register_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["register"])
        .description("Command to register a new group for analytics")
        .handler(analytics_register);

    let analytics_unregister_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["unregister"])
        .description("Command to de-register a group from analytics")
        .handler(analytics_unregister);

    let analytics_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view the membercount analytics of a group")
        .handler(analytics_view);

    let analytics = Command::builder()
        .level(RoLevel::Admin)
        .names(&["analytics"])
        .description("Module to interact with the analytics subsystem")
        .group("Premium")
        .sub_command(analytics_register_cmd)
        .sub_command(analytics_unregister_cmd)
        .sub_command(analytics_view_cmd)
        .handler(analytics_config_view);
    cmds.push(analytics);
}

#[derive(FromArgs)]
pub struct AnalyticsViewArguments {}

pub async fn analytics_config_view(
    ctx: CommandContext,
    _args: AnalyticsViewArguments,
) -> CommandResult {
    let guild = ctx
        .bot
        .database
        .get_guild(ctx.guild_id.unwrap().0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
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
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let registered_groups = guild
        .registered_groups
        .iter()
        .map(ToString::to_string)
        .join("\n");

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Registered Groups")
        .unwrap()
        .description(registered_groups)
        .unwrap()
        .build()
        .unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
