mod custom;
mod delete;
mod group;
mod name;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::blacklist::BlacklistData;

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
        .names(&["blacklist", "bl", "blacklists"])
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

pub async fn blacklist(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    if guild.blacklists.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No blacklists were found associated with this server")
            .build();
        ctx.respond().embeds(&[e])?.exec().await?;
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
            let name = format!("Id: {}", bl.blacklist_id);
            let desc = match &bl.data {
                BlacklistData::User(user) => format!(
                    "Type: {}\nUser Id: {}\nReason: {}",
                    bl.kind(),
                    user,
                    bl.reason
                ),
                BlacklistData::Group(group) => format!(
                    "Type: {}\nGroup Id: {}\nReason: {}",
                    bl.kind(),
                    group,
                    bl.reason
                ),
                BlacklistData::Custom(code) => {
                    format!("Type: {}\nCode: {}\nReason: {}", bl.kind(), code, bl.reason)
                }
            };
            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        }
        pages.push(embed.build());
        page_count += 1;
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
