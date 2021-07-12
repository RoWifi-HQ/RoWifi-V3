mod add;
mod remove;

use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

use add::admin_add;
use remove::admin_remove;

pub fn settings_admin_config() -> Command {
    let admin_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to add roles as RoWifi Admins")
        .names(&["add"])
        .handler(admin_add);

    let admin_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to remove roles from RoWifi Admins")
        .names(&["remove"])
        .handler(admin_remove);

    Command::builder()
        .level(RoLevel::Admin)
        .description("Module to interact with roles that can manage RoWifi")
        .names(&["admin"])
        .sub_command(admin_add_cmd)
        .sub_command(admin_remove_cmd)
        .handler(admin_view)
}

pub async fn admin_view(ctx: CommandContext) -> CommandResult {
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
    for admin_role in guild.settings.admin_roles {
        description.push_str(&format!("- <@&{}>\n", admin_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Admin Roles")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
