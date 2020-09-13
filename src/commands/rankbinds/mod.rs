mod new;
mod modify;
mod delete;

use crate::framework::prelude::*;
use crate::utils::pagination::paginate_embed;
use itertools::Itertools;
use twilight_mention::Mention;
use twilight_embed_builder::EmbedFieldBuilder;

pub use new::*;
pub use modify::*;
pub use delete::*;

pub static RANKBINDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["rankbinds", "rb"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&RANKBINDS_NEW_COMMAND, &RANKBINDS_MODIFY_COMMAND, &RANKBINDS_DELETE_COMMAND],
    group: Some("Binds")
};

pub static RANKBINDS_COMMAND: Command = Command {
    fun: rankbind,
    options: &RANKBINDS_OPTIONS
};

#[command]
pub async fn rankbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => {
            println!("No Guild Found");
            return Ok(())
        }
    };

    if guild.rankbinds.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No rankbinds were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count: usize = 0;
    let distinct_groups = guild.rankbinds.iter().group_by(|r| r.group_id);
    for group in distinct_groups.into_iter() {
        for rbs in   group.1.collect_vec().iter().chunks(12).into_iter() {
            let mut embed = EmbedBuilder::new().default_data().title("Rankbinds").unwrap()
                .description(format!("Group {} | Page {}", group.0, page_count+1)).unwrap();
            for rb in rbs {
                let name = format!("Rank: {}", rb.rank_id);
                let desc = format!("Prefix: {}\nPriority: {}\n Roles: {}", rb.prefix, rb.priority, rb.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>());
                embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
            }
            pages.push(embed.build().unwrap());
            page_count += 1;
        }
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}