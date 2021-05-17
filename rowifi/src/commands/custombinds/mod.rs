mod delete;
mod modify;
mod new;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use twilight_mention::Mention;
use twilight_model::id::RoleId;

use delete::custombinds_delete;
use modify::custombinds_modify;
use new::custombinds_new;

pub fn custombinds_config(cmds: &mut Vec<Command>) {
    let custombinds_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view custombinds")
        .handler(custombinds_view);

    let custombinds_delete_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["delete", "d", "remove"])
        .description("Command to delete a custombind")
        .handler(custombinds_delete);

    let custombinds_modify_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify a custombind")
        .handler(custombinds_modify);

    let custombinds_new_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to create a custombind")
        .handler(custombinds_new);

    let custombinds_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["custombinds", "cb"])
        .description("Module to create, update, view & delete custombinds")
        .group("Binds")
        .sub_command(custombinds_view_cmd)
        .sub_command(custombinds_delete_cmd)
        .sub_command(custombinds_modify_cmd)
        .sub_command(custombinds_new_cmd)
        .handler(custombinds_view);
    cmds.push(custombinds_cmd);
}

#[derive(FromArgs)]
pub struct CustombindsViewArguments {}

pub async fn custombinds_view(
    ctx: CommandContext,
    _args: CustombindsViewArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    if guild.custombinds.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No custombinds were found associated with this server")
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
    for cbs in &guild.custombinds.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Custombinds")
            .description(format!("Page {}", page_count + 1));
        let cbs = cbs.sorted_by_key(|c| c.id);
        for cb in cbs {
            let name = format!("Bind Id: {}", cb.id);
            let roles_str = cb
                .discord_roles
                .iter()
                .map(|r| RoleId(*r as u64).mention().to_string())
                .collect::<String>();
            let nick = if let Some(template) = &cb.template {
                format!("Template: `{}`\n", template)
            } else if let Some(prefix) = &cb.prefix {
                format!("Prefix: `{}`\n", prefix)
            } else {
                String::default()
            };
            let desc = format!(
                "Code: {}\n{}Priority: {}\nRoles: {}",
                cb.code, nick, cb.priority, roles_str
            );
            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
