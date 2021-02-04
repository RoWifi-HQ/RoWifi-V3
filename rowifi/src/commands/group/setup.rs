use framework_new::prelude::*;
use rowifi_models::guild::RoGuild;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct SetupArguments {}

pub async fn setup(ctx: CommandContext, _args: SetupArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let existing_guild = ctx.bot.database.get_guild(guild_id.0).await?;
    let server_roles = ctx.bot.cache.roles(guild_id);

    let verification_role_str = await_reply("Which role would you like to bind as your **unverified/verification** role?\nPlease tag the role for the bot to be able to detect it.", &ctx).await?;
    let verification_role = match parse_role(verification_role_str) {
        Some(v) if server_roles.contains(&RoleId(v)) => v,
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Setup Failed")
                .unwrap()
                .description("Invalid verification role found")
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let verified_role_str = await_reply("Which role would you like to bind as your **verified** role?\nPlease tag the role for the bot to be able to detect it.", &ctx).await?;
    let verified_role = match parse_role(verified_role_str) {
        Some(v) if server_roles.contains(&RoleId(v)) => v,
        _ => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .unwrap()
                .title("Setup Failed")
                .unwrap()
                .description("Invalid verified role found")
                .unwrap()
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let mut replace = false;
    let mut guild = RoGuild {
        id: guild_id.0 as i64,
        verification_role: verification_role as i64,
        verified_role: verified_role as i64,
        ..RoGuild::default()
    };
    if let Some(existing) = existing_guild {
        guild.command_prefix = existing.command_prefix.clone();
        replace = true;
    }

    ctx.bot.database.add_guild(guild, replace).await?;
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Setup Successful!").unwrap()
        .description("Server has been setup successfully. Use `rankbinds new` or `groupbinds new` to start setting up your binds").unwrap()
        .build().unwrap();
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
