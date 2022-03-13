use rowifi_framework::prelude::*;
use rowifi_models::{
    blacklist::Blacklist,
    guild::{BlacklistActionType, GuildType},
    id::{ChannelId, RoleId, UserId},
};

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
            .build();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut db = ctx.bot.database.get().await?;
    let transaction = db.transaction().await?;

    let get = transaction
        .prepare_cached("SELECT premium_owner FROM guilds WHERE guild_id = $1")
        .await?;
    let row = transaction.query_opt(&get, &[&guild_id]).await?;
    if let Some(row) = row {
        let premium_owner: Option<UserId> = row.get("premium_owner");
        if let Some(premium_owner) = premium_owner {
            let remove_user_premium = transaction.prepare_cached("UPDATE users SET premium_servers = array_remove(premium_servers, $2) WHERE discord_id = $1").await?;
            transaction
                .execute(&remove_user_premium, &[&premium_owner, &guild_id])
                .await?;
        }
    }

    let upsert = transaction.prepare_cached(
        r#"INSERT INTO guilds(guild_id, kind, premium_owner, command_prefix, verification_roles, verified_roles, blacklists, disabled_channels, registered_groups, auto_detection, blacklist_action, update_on_join, admin_roles, trainer_roles, bypass_roles, nickname_bypass_roles, log_channel)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17) ON CONFLICT (guild_id) DO UPDATE SET
        kind = $2, premium_owner = $3, command_prefix = $4, verification_roles = $5, verified_roles = $6, blacklists = $7, disabled_channels = $8, registered_groups = $9, auto_detection = $10, blacklist_action = $11, update_on_join = $12, admin_roles = $13, trainer_roles = $14, bypass_roles = $15, nickname_bypass_roles = $16, log_channel = $17"#
    ).await?;
    transaction
        .execute(
            &upsert,
            &[
                &guild_id,                  // guild_id
                &GuildType::Free,           // kind
                &None::<UserId>,            // premium_owner
                &"!",                       // command_prefix
                &Vec::<RoleId>::new(),      // verification_roles
                &Vec::<RoleId>::new(),      // verified_roles
                &Vec::<Blacklist>::new(),   // blacklists
                &Vec::<ChannelId>::new(),   // disabled_channels
                &Vec::<i64>::new(),         // registered_groups
                &false,                     // auto_detection
                &BlacklistActionType::None, // blacklist_action
                &false,                     // update_on_join
                &Vec::<RoleId>::new(),      // admin_roles
                &Vec::<RoleId>::new(),      // trainer_roles
                &Vec::<RoleId>::new(),      // bypass_roles
                &Vec::<RoleId>::new(),      // nickname_bypass_roles
                &None::<ChannelId>,         // log_channel
            ],
        )
        .await?;

    let delete_binds = transaction
        .prepare_cached("DELETE FROM binds WHERE guild_id = $1")
        .await?;
    transaction.execute(&delete_binds, &[&(guild_id)]).await?;

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
        .build();
    ctx.respond().embeds(&[embed])?.exec().await?;

    Ok(())
}
