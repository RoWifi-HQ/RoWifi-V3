mod custom;
mod delete;
mod group;
mod name;

use itertools::Itertools;
use rowifi_framework::prelude::*;

pub use custom::blacklist_custom;
pub use delete::blacklist_delete;
pub use group::blacklist_group;
pub use name::blacklist_name;

pub fn blacklists_config(cmds: &mut Vec<Command>) {
    let blacklist_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view blacklists of a server")
        .handler(blacklist);

    let blacklist_custom_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["custom"])
        .description("Command to add a custom blacklist")
        .handler(blacklist_custom);

    let blacklist_group_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["group"])
        .description("Command to add a group blacklist")
        .handler(blacklist_group);

    let blacklist_name_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["name"])
        .description("Command to add a user blacklist")
        .handler(blacklist_name);

    let blacklist_delete_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["delete", "d", "remove"])
        .description("Command to delete a blacklist")
        .handler(blacklist_delete);

    let blacklist_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["blacklist", "bl"])
        .description("Command to view blacklists of a server")
        .group("Administration")
        .sub_command(blacklist_view_cmd)
        .sub_command(blacklist_custom_cmd)
        .sub_command(blacklist_group_cmd)
        .sub_command(blacklist_name_cmd)
        .sub_command(blacklist_delete_cmd)
        .handler(blacklist);
    cmds.push(blacklist_cmd);
}

#[derive(FromArgs)]
pub struct BlacklistViewArguments {}

pub async fn blacklist(ctx: CommandContext, _args: BlacklistViewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.blacklists.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No blacklists were found associated with this server")
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(e)
            .unwrap()
            .await?;
        return Ok(());
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for bls in &guild.blacklists.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Blacklists")
            .description(format!("Page {}", page_count + 1));
        for bl in bls {
            let name = format!("Type: {:?}", bl.blacklist_type);
            let desc = format!("Id: {}\nReason: {}", bl.id, bl.reason);
            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
