mod misc;
mod update;
mod verify;

use crate::framework::prelude::*;

pub use misc::*;
pub use update::*;
pub use verify::*;

pub static SETTINGS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["settings"],
    desc: Some("Command to view the settings of a server"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &SETTINGS_VERIFICATION_COMMAND,
        &SETTINGS_VERIFIED_COMMAND,
        &UPDATE_JOIN_COMMAND,
        &UPDATE_VERIFY_COMMAND,
        &BLACKLIST_ACTION_COMMAND,
        &TOGGLE_COMMANDS_COMMAND,
        &SETTINGS_PREFIX_COMMAND,
    ],
    group: Some("Administration"),
};

pub static SETTINGS_COMMAND: Command = Command {
    fun: setting,
    options: &SETTINGS_OPTIONS,
};

#[command]
pub async fn setting(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let embed = EmbedBuilder::new()
        .default_data()
        .field(
            EmbedFieldBuilder::new("Tier", guild.settings.guild_type.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Prefix",
                guild.command_prefix.clone().unwrap_or_else(|| "!".into()),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Auto Detection", guild.settings.auto_detection.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Blacklist Action",
                guild.settings.blacklist_action.to_string(),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Update On Join", guild.settings.update_on_join.to_string())
                .unwrap()
                .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Update On Verify",
                guild.settings.update_on_verify.to_string(),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new(
                "Verification Role",
                format!("<@&{}>", guild.verification_role),
            )
            .unwrap()
            .inline(),
        )
        .field(
            EmbedFieldBuilder::new("Verified Role", format!("<@&{}>", guild.verified_role))
                .unwrap()
                .inline(),
        )
        .build()
        .unwrap();

    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
