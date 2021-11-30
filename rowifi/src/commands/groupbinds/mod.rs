mod delete;
mod modify;
mod new;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::bind::{Groupbind, BindType};

pub use delete::groupbinds_delete;
pub use modify::groupbinds_modify;
pub use new::groupbinds_new;

pub fn groupbinds_config(cmds: &mut Vec<Command>) {
    let groupbinds_view_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view groupbinds of the server")
        .handler(groupbinds_view);

    let groupbinds_delete_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["delete", "d", "remove"])
        .description("Command to delete a groupbind")
        .handler(groupbinds_delete);

    let groupbinds_modify_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify an existing groupbind")
        .handler(groupbinds_modify);

    let groupbinds_new_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to create a new groupbind")
        .handler(groupbinds_new);

    let groupbinds_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["groupbinds", "gb"])
        .description("Module to create, update, delete & view groupbinds of the server")
        .group("Binds")
        .sub_command(groupbinds_view_cmd)
        .sub_command(groupbinds_delete_cmd)
        .sub_command(groupbinds_modify_cmd)
        .sub_command(groupbinds_new_cmd)
        .handler(groupbinds_view);
    cmds.push(groupbinds_cmd);
}

pub async fn groupbinds_view(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let groupbinds = ctx.bot.database.query::<Groupbind>("SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY group_id", &[&(guild_id.get() as i64), &BindType::Group]).await?;

    if groupbinds.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No groupbinds were found associated with this server")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for gbs in &groupbinds.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Groupbinds")
            .description(format!("Page {}", page_count + 1));
        let gbs = gbs.sorted_by_key(|g| g.group_id);
        for gb in gbs {
            let name = format!("Group Id: {}", gb.group_id);
            let desc = format!(
                "Template: {}\nPriority: {}\nRoles: {}",
                gb.template,
                gb.priority,
                gb.discord_roles
                    .iter()
                    .map(|r| format!("<@&{}> ", r))
                    .collect::<String>()
            );
            embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
        }
        pages.push(embed.build()?);
        page_count += 1;
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
