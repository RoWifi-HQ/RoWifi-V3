mod new;
mod modify;
mod delete;

use crate::framework::prelude::*;
use crate::utils::pagination::paginate_embed;
use itertools::Itertools;
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_mention::Mention;

pub use new::*;
pub use modify::*;
pub use delete::*;

pub static GROUPBINDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["groupbinds", "gb"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&GROUPBINDS_NEW_COMMAND, &GROUPBINDS_MODIFY_COMMAND, &GROUPBINDS_DELETE_COMMAND],
    group: Some("Binds")
};

pub static GROUPBINDS_COMMAND: Command = Command {
    fun: groupbind,
    options: &GROUPBINDS_OPTIONS
};

#[command]
pub async fn groupbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    if guild.groupbinds.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No groupbinds were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for binds in guild.groupbinds.iter().chunks(12).into_iter() {
        let mut embed = EmbedBuilder::new().default_data().title("Groupbinds").unwrap()
                .description(format!("Page {}", page_count+1)).unwrap();
        for gb in binds {
            let name = format!("Group Id: {}", gb.group_id);
            let desc = format!("Roles: {}", gb.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>());
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}