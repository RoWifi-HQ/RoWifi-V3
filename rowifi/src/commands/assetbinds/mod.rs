mod delete;
mod modify;
mod new;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use twilight_mention::Mention;
use twilight_model::id::RoleId;

pub use delete::assetbinds_delete;
pub use modify::assetbinds_modify;
pub use new::assetbinds_new;

pub fn assetbinds_config(cmds: &mut Vec<Command>) {
    let assetbinds_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view the assetbinds of the server")
        .handler(assetbind);

    let assetbinds_new_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to add a new assetbind")
        .handler(assetbinds_new);

    let assetbinds_modify_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify an existing assetbind")
        .handler(assetbinds_modify);

    let assetbinds_delete_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["delete", "d", "remove"])
        .description("Commmand to delete an assetbind")
        .handler(assetbinds_delete);

    let assetbinds_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["assetbinds", "ab"])
        .description("Module to create, update, delete and view the assetbinds")
        .group("Binds")
        .sub_command(assetbinds_view_cmd)
        .sub_command(assetbinds_new_cmd)
        .sub_command(assetbinds_modify_cmd)
        .sub_command(assetbinds_delete_cmd)
        .handler(assetbind);
    cmds.push(assetbinds_cmd);
}

#[derive(FromArgs)]
pub struct AssetbindArguments {}

pub async fn assetbind(ctx: CommandContext, _args: AssetbindArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.assetbinds.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description("No assetbinds were found associated with this server")
            .unwrap()
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
    for abs in &guild.assetbinds.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("AssetBinds")
            .unwrap()
            .description(format!("Page {}", page_count + 1))
            .unwrap();
        let abs = abs.sorted_by_key(|a| a.id);
        for ab in abs {
            let name = format!("Id: {}", ab.id);
            let roles_str = ab
                .discord_roles
                .iter()
                .map(|r| RoleId(*r as u64).mention().to_string())
                .collect::<String>();
            let nick = match &ab.template {
                Some(template) => format!("Template: {}\n", template),
                None => String::default(),
            };
            let desc = format!(
                "Type: {}\n{}Priority: {}\nRoles: {}",
                ab.asset_type, nick, ab.priority, roles_str
            );
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }

    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
