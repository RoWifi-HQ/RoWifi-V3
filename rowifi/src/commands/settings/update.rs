use mongodb::bson::doc;
use rowifi_framework::prelude::*;

use super::ToggleOption;

#[derive(FromArgs)]
pub struct UpdateOnJoinArguments {
    #[arg(help = "Option to toggle the `Update on Join` setting")]
    pub option: ToggleOption,
}

pub async fn update_on_join(ctx: CommandContext, args: UpdateOnJoinArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let option = args.option;
    let (option, desc) = match option {
        ToggleOption::Enable => (true, "Update on Join has succesfully been enabled"),
        ToggleOption::Disable => (false, "Update on Join has successfully been disabled"),
    };

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$set": {"Settings.UpdateOnJoin": option}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Settings Modification Successful")
        .description(desc)
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description(format!(
            "Settings Modification: Update On Join - {} -> {}",
            guild.settings.update_on_join, option
        ))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
