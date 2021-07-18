use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

pub async fn reset(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    ctx.bot.database.get_guild(guild_id.0).await?;
    let guild = RoGuild {
        id: guild_id.0 as i64,
        ..RoGuild::default()
    };

    ctx.bot.database.add_guild(&guild, true).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Reset Successful!")
        .description("Your settings & bind configurations have been reset")
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    Ok(())
}
