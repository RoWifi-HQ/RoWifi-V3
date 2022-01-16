mod admin;
mod bypass;
mod functional;
mod log;
mod misc;
mod nickname_bypass;
mod trainer;
mod update;
mod verify;

use itertools::Itertools;
use rowifi_framework::prelude::*;

use admin::{admin_add, admin_remove, admin_set, admin_view};
use bypass::{bypass_add, bypass_remove, bypass_set, bypass_view};
use functional::functional;
use log::log_channel;
use misc::{blacklist_action, settings_prefix, toggle_commands};
use nickname_bypass::{
    nickname_bypass_add, nickname_bypass_remove, nickname_bypass_set, nickname_bypass_view,
};
use trainer::{trainer_add, trainer_remove, trainer_set, trainer_view};
use update::update_on_join;
use verify::{
    settings_verification_add, settings_verification_remove, settings_verified_add,
    settings_verified_remove,
};

pub fn settings_config(cmds: &mut Vec<Command>) {
    let settings_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to interact with the settings of the server")
        .handler(settings_view);

    let settings_blacklist_action_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["blacklist-action", "bl-action"])
        .description("Command to set the blacklist action setting")
        .handler(blacklist_action);

    let settings_toggle_commands_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["commands", "command-channel", "command"])
        .description("Command to toggle command usage in a channel")
        .handler(toggle_commands);

    let settings_prefix_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["prefix"])
        .description("Command to change the bot's prefix in the server")
        .handler(settings_prefix);

    let update_on_join_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["update-on-join", "uoj"])
        .description("Command to toggle the `Update On Join` setting in the server")
        .handler(update_on_join);

    let settings_verification_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add verification roles")
        .handler(settings_verification_add);

    let settings_verification_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove verification roles")
        .handler(settings_verification_remove);

    let settings_verification_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["verification", "unverified"])
        .description("Module to add or remove verification roles")
        .sub_command(settings_verification_add_cmd)
        .sub_command(settings_verification_remove_cmd)
        .handler(settings_view);

    let settings_verified_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add verified roles")
        .handler(settings_verified_add);

    let settings_verified_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove verified roles")
        .handler(settings_verified_remove);

    let settings_verified_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["verified"])
        .description("Module to add or remove verified roles")
        .sub_command(settings_verified_add_cmd)
        .sub_command(settings_verified_remove_cmd)
        .handler(settings_view);

    let admin_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add custom admin roles")
        .handler(admin_add);

    let admin_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove custom admin roles")
        .handler(admin_remove);

    let admin_set_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["set"])
        .description("Command to set custom admin roles")
        .handler(admin_set);

    let admin_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view custom admin roles")
        .handler(admin_view);

    let settings_admin_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["admin"])
        .description("Module to interact with the custom admin roles")
        .sub_command(admin_add_cmd)
        .sub_command(admin_remove_cmd)
        .sub_command(admin_set_cmd)
        .sub_command(admin_view_cmd)
        .handler(admin_view);

    let trainer_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add custom trainer roles")
        .handler(trainer_add);

    let trainer_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove custom trainer roles")
        .handler(trainer_remove);

    let trainer_set_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["set"])
        .description("Command to set custom trainer roles")
        .handler(trainer_set);

    let trainer_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view custom trainer roles")
        .handler(trainer_view);

    let settings_trainer_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["trainer"])
        .description("Module to interact with the custom trainer roles")
        .sub_command(trainer_add_cmd)
        .sub_command(trainer_remove_cmd)
        .sub_command(trainer_set_cmd)
        .sub_command(trainer_view_cmd)
        .handler(trainer_view);

    let bypass_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add custom bypass roles")
        .handler(bypass_add);

    let bypass_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove custom bypass roles")
        .handler(bypass_remove);

    let bypass_set_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["set"])
        .description("Command to set custom bypass roles")
        .handler(bypass_set);

    let bypass_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view custom bypass roles")
        .handler(bypass_view);

    let settings_bypass_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["bypass"])
        .description("Module to interact with the custom bypass roles")
        .sub_command(bypass_add_cmd)
        .sub_command(bypass_remove_cmd)
        .sub_command(bypass_set_cmd)
        .sub_command(bypass_view_cmd)
        .handler(bypass_view);

    let nickname_bypass_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add"])
        .description("Command to add custom nickname bypass roles")
        .handler(nickname_bypass_add);

    let nickname_bypass_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["remove"])
        .description("Command to remove custom nickname bypass roles")
        .handler(nickname_bypass_remove);

    let nickname_bypass_set_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["set"])
        .description("Command to set custom nickname bypass roles")
        .handler(nickname_bypass_set);

    let nickname_bypass_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Module to view custom nickname bypass roles")
        .handler(nickname_bypass_view);

    let settings_nickname_bypass_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["nickname-bypass", "nb"])
        .description("Module to interact with the custom nickname bypass roles")
        .sub_command(nickname_bypass_add_cmd)
        .sub_command(nickname_bypass_remove_cmd)
        .sub_command(nickname_bypass_set_cmd)
        .sub_command(nickname_bypass_view_cmd)
        .handler(nickname_bypass_view);

    let log_channel_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["log-channel", "logchannel", "lc"])
        .description("Command to interact with the channel where RoWifi sends logs")
        .handler(log_channel);

    let functional_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["functional"])
        .description("Command to change the RoWifi permissions of a discord role")
        .handler(functional);

    let settings_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["settings", "setting"])
        .description("Module to interact with the settings of the server")
        .group("Administration")
        .sub_command(settings_view_cmd)
        .sub_command(settings_blacklist_action_cmd)
        .sub_command(settings_toggle_commands_cmd)
        .sub_command(settings_prefix_cmd)
        .sub_command(update_on_join_cmd)
        .sub_command(settings_verification_cmd)
        .sub_command(settings_verified_cmd)
        .sub_command(settings_admin_cmd)
        .sub_command(settings_trainer_cmd)
        .sub_command(settings_bypass_cmd)
        .sub_command(settings_nickname_bypass_cmd)
        .sub_command(log_channel_cmd)
        .sub_command(functional_cmd)
        .handler(settings_view);
    cmds.push(settings_cmd);
}

pub async fn settings_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;
    let mut verification_roles = guild
        .verification_roles
        .iter()
        .map(|r| format!("<@&{}>", r))
        .join(" ");
    if verification_roles.is_empty() {
        verification_roles = "None".into();
    }
    let mut verified_roles = guild
        .verified_roles
        .iter()
        .map(|r| format!("<@&{}>", r))
        .join(" ");
    if verified_roles.is_empty() {
        verified_roles = "None".into();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .field(EmbedFieldBuilder::new("Tier", guild.kind.to_string()).inline())
        .field(EmbedFieldBuilder::new("Prefix", &guild.command_prefix).inline())
        .field(EmbedFieldBuilder::new("Auto Detection", guild.auto_detection.to_string()).inline())
        .field(
            EmbedFieldBuilder::new("Blacklist Action", guild.blacklist_action.to_string()).inline(),
        )
        .field(EmbedFieldBuilder::new("Update On Join", guild.update_on_join.to_string()).inline())
        .field(EmbedFieldBuilder::new("Verification Role", verification_roles).inline())
        .field(EmbedFieldBuilder::new("Verified Role", verified_roles).inline())
        .build()
        .unwrap();

    ctx.respond().embeds(&[embed])?.exec().await?;
    Ok(())
}

pub enum ToggleOption {
    Enable,
    Disable,
}

impl FromArg for ToggleOption {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match arg.to_ascii_lowercase().as_str() {
            "enable" | "on" => Ok(ToggleOption::Enable),
            "disable" | "off" => Ok(ToggleOption::Disable),
            _ => Err(ParseError("one of `enable` `disable` `on` `off`")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match &option.value {
            CommandOptionValue::String(value) => value.to_string(),
            CommandOptionValue::Integer(value) => value.to_string(),
            _ => unreachable!("ToggleOption unreached"),
        };

        Self::from_arg(&arg)
    }
}
