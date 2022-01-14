mod delete;
mod modify;
mod new;

use itertools::Itertools;
use rowifi_framework::prelude::*;
use rowifi_models::bind::{BindType, Rankbind};

pub use delete::*;
pub use modify::*;
pub use new::*;

pub fn rankbinds_config(cmds: &mut Vec<Command>) {
    let rankbinds_new_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to add a new rankbind")
        .handler(rankbinds_new);

    let rankbinds_modify_priority_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["priority"])
        .description("Command to modify the priority of a rankbind")
        .handler(rb_modify_priority);

    let rankbinds_modify_template_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["priority"])
        .description("Command to modify the template of a rankbind")
        .handler(rb_modify_template);

    let rankbinds_add_roles_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["add-roles"])
        .description("Command to add roles to a rankbind")
        .handler(rb_add_roles);

    let rankbinds_remove_roles_cmd = Command::builder()
        .level(RoLevel::Admin)
        .names(&["priority"])
        .description("Command to remove roles from a rankbind")
        .handler(rb_remove_roles);

    let rankbinds_modify_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify an existing rankbind")
        .sub_command(rankbinds_modify_priority_cmd)
        .sub_command(rankbinds_modify_template_cmd)
        .sub_command(rankbinds_add_roles_cmd)
        .sub_command(rankbinds_remove_roles_cmd)
        .handler(rankbinds_view);

    let rankbinds_delete_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["delete", "d", "remove"])
        .description("Command to delete an existing rankbind")
        .handler(rankbinds_delete);

    let rankbinds_view_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["view"])
        .description("Command to view all rankbinds")
        .handler(rankbinds_view);

    let rankbinds = Command::builder()
        .level(RoLevel::Admin)
        .names(&["rankbinds", "rb"])
        .description("Command to view the rankbinds")
        .group("Binds")
        .sub_command(rankbinds_new_command)
        .sub_command(rankbinds_modify_command)
        .sub_command(rankbinds_delete_command)
        .sub_command(rankbinds_view_command)
        .handler(rankbinds_view);

    cmds.push(rankbinds);
}

pub async fn rankbinds_view(ctx: CommandContext) -> Result<(), RoError> {
    let guild_id = ctx.guild_id.unwrap();
    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2 ORDER BY group_id ASC, group_rank_id ASC",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

    if rankbinds.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No rankbinds were found associated with this server")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let mut pages = Vec::new();
    let mut page_count: usize = 0;
    let distinct_groups = rankbinds.iter().group_by(|r| r.group_id);
    for group in &distinct_groups {
        for rbs in &group.1.collect_vec().iter().chunks(12) {
            let mut embed = EmbedBuilder::new()
                .default_data()
                .title("Rankbinds")
                .description(format!("Group {} | Page {}", group.0, page_count + 1));
            let rbs = rbs.sorted_by_key(|r| r.group_rank_id);
            for rb in rbs {
                let name = format!("Rank: {}", rb.group_rank_id);
                let desc = format!(
                    "Template: `{}`\nPriority: {}\n Roles: {}",
                    rb.template,
                    rb.priority,
                    rb.discord_roles
                        .iter()
                        .map(|r| format!("<@&{}> ", r))
                        .collect::<String>()
                );
                embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
            }
            pages.push(embed.build()?);
            page_count += 1;
        }
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
