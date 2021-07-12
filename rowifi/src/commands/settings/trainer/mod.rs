mod add;
mod remove;

use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;

use add::trainer_add;
use remove::trainer_remove;

pub fn settings_trainer_config() -> Command {
    let trainer_add_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to add roles as RoWifi Trainers")
        .names(&["add"])
        .handler(trainer_add);

    let trainer_remove_cmd = Command::builder()
        .level(RoLevel::Admin)
        .description("Command to remove roles from RoWifi Trainers")
        .names(&["remove"])
        .handler(trainer_remove);

    Command::builder()
        .level(RoLevel::Admin)
        .description("Module to interact with roles that can interact with trainer commands")
        .names(&["trainer"])
        .sub_command(trainer_add_cmd)
        .sub_command(trainer_remove_cmd)
        .handler(trainer_view)
}

pub async fn trainer_view(ctx: CommandContext) -> CommandResult {
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
    for trainer_role in guild.settings.trainer_roles {
        description.push_str(&format!("- <@&{}>\n", trainer_role));
    }

    if description.is_empty() {
        description = "None".to_string();
    }

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Trainer Roles")
        .description(description)
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
