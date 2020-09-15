mod new;
mod modify;
mod delete;

use crate::framework::prelude::*;
use crate::utils::pagination::paginate_embed;
use itertools::Itertools;
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_mention::Mention;

use new::*;
use modify::*;
use delete::*;

pub static CUSTOMBINDS_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["custombinds", "cb"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[&CUSTOMBINDS_NEW_COMMAND, &CUSTOMBINDS_MODIFY_COMMAND, &CUSTOMBINDS_DELETE_COMMAND],
    group: Some("Binds")
};

pub static CUSTOMBINDS_COMMAND: Command = Command {
    fun: custombind,
    options: &CUSTOMBINDS_OPTIONS
};

#[command]
pub async fn custombind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    if guild.custombinds.is_empty() {
        let e = EmbedBuilder::new().default_data().title("Bind Viewing Failed").unwrap().color(Color::Red as u32).unwrap()
            .description("No custombinds were found associated with this server").unwrap().build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await;
        return Ok(())
    }

    let mut pages = Vec::new();
    let mut page_count = 0;
    for binds in guild.custombinds.iter().chunks(12).into_iter() {
        let mut embed = EmbedBuilder::new().default_data().title("Custombinds").unwrap()
                .description(format!("Page {}", page_count+1)).unwrap();
        for cb in binds {
            let name = format!("Bind Id: {}", cb.id);
            let roles_str = cb.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>();
            let desc = format!("Code: {}\nPrefix: {}\nPriority: {}\nRoles: {}", cb.code, cb.prefix, cb.priority, roles_str);
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}