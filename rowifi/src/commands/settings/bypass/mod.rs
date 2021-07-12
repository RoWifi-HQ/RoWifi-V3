mod add;
mod remove;

use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

use add::{bypass_add, nickname_bypass_add};
use remove::{bypass_remove, nickname_bypass_remove};

pub fn settings_bypass_config() -> (Command, Command) {
    let bypass_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to add roles as RoWifi Bypass")
        .names(&["add"])
        .handler(bypass_add);

    let nickname_bypass_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to add roles as RoWifi Nickname Bypass")
        .names(&["add"])
        .handler(nickname_bypass_add);

    let bypass_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to remove roles from RoWifi Bypass")
        .names(&["remove"])
        .handler(bypass_remove);

    let nickname_bypass_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to remove roles from RoWifi Nickname Bypass")
        .names(&["remove"])
        .handler(nickname_bypass_remove);

    let bypass_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Module to interact with custom RoWifi Bypass roles")
        .names(&["bypass"])
        .sub_command(bypass_add_cmd)
        .sub_command(bypass_remove_cmd)
        .handler(bypass_view);

    let nickname_bypass_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Module to interact with custom RoWifi Nickname Bypass roles")
        .names(&["nickname-bypass", "nb"])
        .sub_command(nickname_bypass_add_cmd)
        .sub_command(nickname_bypass_remove_cmd)
        .handler(nickname_bypass_view);

    (bypass_cmd, nickname_bypass_cmd)
}

pub async fn bypass_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let mut description = String::new();
    for bypass_role in guild.settings.bypass_roles {
        description.push_str(&format!("- <@&{}>\n", bypass_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bypass Roles")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}

pub async fn nickname_bypass_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let mut description = String::new();
    for admin_role in guild.settings.nickname_bypass_roles {
        description.push_str(&format!("- <@&{}>\n", admin_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Nickname Bypass Roles")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
