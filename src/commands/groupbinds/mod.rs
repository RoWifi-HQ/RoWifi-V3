mod delete;
mod modify;
mod new;

use crate::framework::prelude::*;
use crate::utils::misc::paginate_embed;
use itertools::Itertools;
use twilight_mention::Mention;

pub use delete::*;
pub use modify::*;
pub use new::*;

pub static GROUPBINDS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["groupbinds", "gb"],
    desc: Some("Command to view groupbinds"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &GROUPBINDS_NEW_COMMAND,
        &GROUPBINDS_MODIFY_COMMAND,
        &GROUPBINDS_DELETE_COMMAND,
    ],
    group: Some("Binds"),
};

pub static GROUPBINDS_COMMAND: Command = Command {
    fun: groupbind,
    options: &GROUPBINDS_OPTIONS,
};

#[command]
pub async fn groupbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    if guild.groupbinds.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description("No groupbinds were found associated with this server")
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
    let mut page_count = 0;
    for gbs in &guild.groupbinds.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Groupbinds")
            .unwrap()
            .description(format!("Page {}", page_count + 1))
            .unwrap();
        let gbs = gbs.sorted_by_key(|g| g.group_id);
        for gb in gbs {
            let name = format!("Group Id: {}", gb.group_id);
            let desc = format!(
                "Roles: {}",
                gb.discord_roles
                    .iter()
                    .map(|r| RoleId(*r as u64).mention().to_string())
                    .collect::<String>()
            );
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}
