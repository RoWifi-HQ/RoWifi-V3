use crate::framework::prelude::*;
use rowifi_models::guild::RoGuild;

pub static SETUP_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["setup"],
    desc: Some("Command to set up the server. May also be used to reset all configurations."),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[],
    group: Some("Administration"),
};

pub static SETUP_COMMAND: Command = Command {
    fun: setup,
    options: &SETUP_OPTIONS,
};

#[command]
pub async fn setup(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let existing_guild = ctx.database.get_guild(msg.guild_id.unwrap().0).await?;
    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());

    let verification_role_str = await_reply("Which role would you like to bind as your **unverified/verification** role?\nPlease tag the role for the bot to be able to detect it.", ctx, msg).await?;
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let verified_role_str = await_reply("Which role would you like to bind as your **verified** role?\nPlease tag the role for the bot to be able to detect it.", ctx, msg).await?;
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
            let _ = ctx
                .http
                .create_message(msg.channel_id)
                .embed(embed)
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let mut replace = false;
    let mut guild = RoGuild {
        id: msg.guild_id.unwrap().0 as i64,
        verification_role: verification_role as i64,
        verified_role: verified_role as i64,
        ..RoGuild::default()
    };
    if let Some(existing) = existing_guild {
        guild.command_prefix = existing.command_prefix.clone();
        replace = true;
    }

    ctx.database.add_guild(guild, replace).await?;
    let embed = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Setup Successful!").unwrap()
        .description("Server has been setup successfully. Use `rankbinds new` or `groupbinds new` to start setting up your binds").unwrap()
        .build().unwrap();
    ctx.http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await?;
    Ok(())
}
