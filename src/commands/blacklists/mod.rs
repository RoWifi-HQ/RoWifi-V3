mod name;
mod group;
mod custom;
mod delete;

use crate::framework::prelude::*;
use crate::utils::pagination::paginate_embed;
use itertools::Itertools;
use twilight_embed_builder::EmbedFieldBuilder;

pub use name::*;
pub use group::*;
pub use custom::*;
pub use delete::*;

pub static BLACKLISTS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["blacklists", "bl"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&BLACKLISTS_NAME_COMMAND, &BLACKLISTS_GROUP_COMMAND, &BLACKLISTS_CUSTOM_COMMAND, &BLACKLISTS_DELETE_COMMAND],
    group: Some("Administration")
};

pub static BLACKLISTS_COMMAND: Command = Command {
    fun: blacklist,
    options: &BLACKLISTS_OPTIONS
};

#[command]
pub async fn blacklist(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    if guild.blacklists.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No blacklists were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for bls in guild.blacklists.iter().chunks(12).into_iter() {
        let mut embed = EmbedBuilder::new().default_data().title("Blacklists").unwrap()
                .description(format!("Page {}", page_count+1)).unwrap();
        for bl in bls {
            let name = format!("Type: {:?}", bl.blacklist_type);
            let desc = format!("Id: {}\nReason: {}", bl.id, bl.reason);
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}