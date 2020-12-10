mod delete;
mod modify;
mod new;

use rowifi_framework::prelude::*;
use itertools::Itertools;
use twilight_mention::Mention;

pub use delete::*;
pub use modify::*;
pub use new::*;

pub static RANKBINDS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["rankbinds", "rb"],
    desc: Some("Command to view the rankbinds"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &RANKBINDS_NEW_COMMAND,
        &RANKBINDS_MODIFY_COMMAND,
        &RANKBINDS_DELETE_COMMAND,
    ],
    group: Some("Binds"),
};

pub static RANKBINDS_COMMAND: Command = Command {
    fun: rankbind,
    options: &RANKBINDS_OPTIONS,
};

#[command]
pub async fn rankbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    if guild.rankbinds.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description("No rankbinds were found associated with this server")
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(e)
            .unwrap()
            .await;
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
                .unwrap()
                .description(format!("Group {} | Page {}", group.0, page_count + 1))
                .unwrap();
            let rbs = rbs.sorted_by_key(|r| r.rank_id);
            for rb in rbs {
                let name = format!("Rank: {}", rb.rank_id);
                let desc = format!(
                    "Prefix: {}\nPriority: {}\n Roles: {}",
                    rb.prefix,
                    rb.priority,
                    rb.discord_roles
                        .iter()
                        .map(|r| RoleId(*r as u64).mention().to_string())
                        .collect::<String>()
                );
                embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
            }
            pages.push(embed.build().unwrap());
            page_count += 1;
        }
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}
