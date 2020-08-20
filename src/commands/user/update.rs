use crate::framework::prelude::*;
use twilight::model::id::UserId;
use twilight_embed_builder::EmbedFooterBuilder;
use std::sync::Arc;

pub static UPDATE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["update", "getroles"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static UPDATE_COMMAND: Command = Command {
    fun: update,
    options: &UPDATE_OPTIONS
};

#[command]
pub async fn update(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let start = chrono::Utc::now().timestamp_millis();
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };
    let server = ctx.cache.guild(guild_id).await.unwrap();

    let user_id = match args.next() {
        Some(s) => match parse_username(s).await {
            Some(id) => UserId(id),
            None => msg.author.id
        },
        None => msg.author.id
    };

    let member = match ctx.member(guild_id, user_id).await? {
        Some(m) => m,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed").unwrap()
                .description("No such member was found").unwrap()
                .color(Color::Red as u32).unwrap()
                .build().unwrap();
            let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;
            return Ok(())
        }
    };

    //Check for server owner
    if server.owner_id.0 == member.user.id.0 {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Update Failed").unwrap()
            .description("Due to discord limitations, I cannot update the server owner").unwrap()
            .color(Color::Red as u32).unwrap()
            .build().unwrap();
        let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;
        return Ok(())
    }

    //Handle role position check

    //Check for bypass role
    let bypass = ctx.cache.bypass_roles(guild_id).await; 
    if let Some(bypass_role) = &bypass.0 {
        if member.roles.contains(bypass_role) {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed").unwrap()
                .description("I cannot update users with the `RoWifi Bypass` role").unwrap()
                .color(Color::Red as u32).unwrap()
                .build().unwrap();
            let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;
            return Ok(())
        }
    }

    let user = match ctx.database.get_user(msg.author.id.0).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed").unwrap()
                .description("User was not verified. Please ask him/her to verify themselves").unwrap()
                .color(Color::Red as u32).unwrap()
                .build().unwrap();
            let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;
            return Ok(())
        }
    };

    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .title("Update Failed").unwrap()
                .description("Server is not set up. Please ask the server owner to set up the server.").unwrap()
                .color(Color::Red as u32).unwrap()
                .build().unwrap();
            let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;
            return Ok(())
        }
    }; 
    let guild_roles = ctx.cache.roles(guild_id).await;

    let (added_roles, removed_roles, disc_nick) = user.update(Arc::clone(&ctx.http), member, Arc::clone(&ctx.roblox), server, &guild, guild_roles).await?;
    let end = chrono::Utc::now().timestamp_millis();
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Update").unwrap()
        .update_log(added_roles, removed_roles, &disc_nick)
        .color(Color::DarkGreen as u32).unwrap()
        .footer(EmbedFooterBuilder::new(format!("RoWifi | Executed in {} ms", (end - start))).unwrap())
        .build().unwrap();
    let _ = ctx.http.as_ref().create_message(msg.channel_id).embed(embed).unwrap().await;

    Ok(())
}