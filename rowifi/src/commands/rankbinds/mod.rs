mod delete;
mod modify;
mod new;

use itertools::Itertools;
use rowifi_framework::prelude::*;

pub use delete::*;
pub use modify::*;
pub use new::*;

pub fn rankbinds_config(cmds: &mut Vec<Command>) {
    let rankbinds_new_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["new"])
        .description("Command to add a new rankbind")
        .handler(rankbinds_new);

    let rankbinds_modify_command = Command::builder()
        .level(RoLevel::Admin)
        .names(&["modify", "m"])
        .description("Command to modify an existing rankbind")
        .handler(rankbinds_modify);

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

#[derive(FromArgs)]
pub struct RankbindArguments {}

pub async fn rankbinds_view(ctx: CommandContext, _args: RankbindArguments) -> Result<(), RoError> {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.rankbinds.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .color(Color::Red as u32)
            .description("No rankbinds were found associated with this server")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let mut pages = Vec::new();
    let mut page_count: usize = 0;
    let distinct_groups = guild.rankbinds.iter().group_by(|r| r.group_id);
    for group in &distinct_groups {
        for rbs in &group.1.collect_vec().iter().chunks(12) {
            let mut embed = EmbedBuilder::new()
                .default_data()
                .title("Rankbinds")
                .description(format!("Group {} | Page {}", group.0, page_count + 1));
            let rbs = rbs.sorted_by_key(|r| r.rank_id);
            for rb in rbs {
                let name = format!("Rank: {}", rb.rank_id);
                let nick = if let Some(template) = &rb.template {
                    format!("Template: `{}`\n", template)
                } else if let Some(prefix) = &rb.prefix {
                    format!("Prefix: `{}`\n", prefix)
                } else {
                    String::default()
                };
                let desc = format!(
                    "{}Priority: {}\n Roles: {}",
                    nick,
                    rb.priority,
                    rb.discord_roles
                        .iter()
                        .map(|r| format!("<@&{}> ", r))
                        .collect::<String>()
                );
                embed = embed.field(EmbedFieldBuilder::new(name, desc).inline().build());
            }
            pages.push(embed.build().unwrap());
            page_count += 1;
        }
    }
    paginate_embed(&ctx, pages, page_count).await?;
    Ok(())
}
