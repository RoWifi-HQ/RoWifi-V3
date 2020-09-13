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

pub static ASSETBINDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["assetbinds", "ab"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&ASSETBINDS_NEW_COMMAND, &ASSETBINDS_MODIFY_COMMAND, &ASSETBINDS_DELETE_COMMAND],
    group: Some("Binds")
};

pub static ASSETBINDS_COMMAND: Command = Command {
    fun: assetbind,
    options: &ASSETBINDS_OPTIONS
};

#[command]
pub async fn assetbind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    if guild.assetbinds.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No assetbinds were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for binds in guild.assetbinds.iter().chunks(12).into_iter() {
        let mut embed = EmbedBuilder::new().default_data().title("AssetBinds").unwrap()
                .description(format!("Page {}", page_count+1)).unwrap();
        for ab in binds {
            let name = format!("Id: {}", ab.id);
            let roles_str = ab.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>();
            let desc = format!("Type: {}\nRoles: {}", ab.asset_type, roles_str);
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }

    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}