use rowifi_framework::prelude::*;
use rowifi_models::guild::RoGuild;

pub async fn reset(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

    let confirmation = await_confirmation(
        "Are you sure you would like to reset all binds & configurations?",
        &ctx,
    )
    .await?;
    if !confirmation {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Reset was cancelled!")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;
    let delete_statement = transaction
        .prepare_cached("DELETE FROM guilds WHERE guild_id = $1")
        .await?;
    transaction
        .execute(&delete_statement, &[&(guild_id)])
        .await?;
    let insert_statement = transaction.prepare_cached("INSERT INTO guilds(guild_id, command_prefix, kind, blacklist_action) VALUES($1, $2, $3, $4)").await?;
    let guild = RoGuild::new(guild_id);
    transaction
        .execute(
            &insert_statement,
            &[
                &(guild_id),
                &guild.command_prefix,
                &guild.kind,
                &guild.blacklist_action,
            ],
        )
        .await?;
    let delete_binds = transaction
        .prepare_cached("DELETE FROM binds WHERE guild_id = $1")
        .await?;
    transaction
        .execute(&delete_binds, &[&(guild_id)])
        .await?;
    transaction.commit().await?;

    ctx.bot.admin_roles.remove(&guild_id);
    ctx.bot.trainer_roles.remove(&guild_id);
    ctx.bot.bypass_roles.remove(&guild_id);
    ctx.bot.nickname_bypass_roles.remove(&guild_id);
    ctx.bot.log_channels.remove(&guild_id);

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Reset Successful!")
        .description("Your settings & bind configurations have been reset")
        .build()
        .unwrap();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
