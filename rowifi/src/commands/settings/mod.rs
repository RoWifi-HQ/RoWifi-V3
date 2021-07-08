mod misc;
mod update;
mod verify;

use rowifi_framework::prelude::*;

pub use misc::{blacklist_action, settings_prefix, toggle_commands};
pub use update::update_on_join;
pub use verify::*;

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

    let settings_verification_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["verification"])
        .description("Command to change the verification role")
        .handler(settings_verification);

    let settings_verified_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["verified"])
        .description("Command to change the verified role")
        .handler(settings_verified);

    let settings_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["settings"])
        .description("Module to interact with the settings of the server")
        .group("Administration")
        .sub_command(settings_view_cmd)
        .sub_command(settings_blacklist_action_cmd)
        .sub_command(settings_toggle_commands_cmd)
        .sub_command(settings_prefix_cmd)
        .sub_command(update_on_join_cmd)
        .sub_command(settings_verification_cmd)
        .sub_command(settings_verified_cmd)
        .handler(settings_view);
    cmds.push(settings_cmd);
}

pub async fn settings_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let embed = EmbedBuilder::new()
        .default_data()
        .field(EmbedFieldBuilder::new("Tier", guild.settings.guild_type.to_string()).inline())
        .field(
            EmbedFieldBuilder::new(
                "Prefix",
                guild.command_prefix.clone().unwrap_or_else(|| "!".into()),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Auto Detection", guild.settings.auto_detection.to_string())
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Blacklist Action",
                guild.settings.blacklist_action.to_string(),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Update On Join", guild.settings.update_on_join.to_string())
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Verification Role",
                format!("<@&{}>", guild.verification_role),
            )
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Verified Role", format!("<@&{}>", guild.verified_role))
                .inline(),
        )
        .build()
        .unwrap();

    ctx.respond().embed(embed).await?;
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
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("ToggleOption unreached"),
        };

        Self::from_arg(&arg)
    }
}
