mod delete;
mod modify;
mod new;

use crate::framework::prelude::*;
use crate::utils::misc::paginate_embed;
use itertools::Itertools;
use twilight_mention::Mention;

use delete::CUSTOMBINDS_DELETE_COMMAND;
use modify::CUSTOMBINDS_MODIFY_COMMAND;
use new::CUSTOMBINDS_NEW_COMMAND;

pub static CUSTOMBINDS_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["custombinds", "cb"],
    desc: Some("Command to view the custom binds"),
    usage: None,
    examples: &[],
    min_args: 0,
    hidden: false,
    sub_commands: &[
        &CUSTOMBINDS_NEW_COMMAND,
        &CUSTOMBINDS_MODIFY_COMMAND,
        &CUSTOMBINDS_DELETE_COMMAND,
    ],
    group: Some("Binds"),
};

pub static CUSTOMBINDS_COMMAND: Command = Command {
    fun: custombind,
    options: &CUSTOMBINDS_OPTIONS,
};

#[command]
pub async fn custombind(ctx: &Context, msg: &Message, _args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    if guild.custombinds.is_empty() {
        let e = EmbedBuilder::new()
            .default_data()
            .title("Bind Viewing Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description("No custombinds were found associated with this server")
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
    for cbs in &guild.custombinds.iter().chunks(12) {
        let mut embed = EmbedBuilder::new()
            .default_data()
            .title("Custombinds")
            .unwrap()
            .description(format!("Page {}", page_count + 1))
            .unwrap();
        let cbs = cbs.sorted_by_key(|c| c.id);
        for cb in cbs {
            let name = format!("Bind Id: {}", cb.id);
            let roles_str = cb
                .discord_roles
                .iter()
                .map(|r| RoleId(*r as u64).mention().to_string())
                .collect::<String>();
            let desc = format!(
                "Code: {}\nPrefix: {}\nPriority: {}\nRoles: {}",
                cb.code, cb.prefix, cb.priority, roles_str
            );
            embed = embed.field(EmbedFieldBuilder::new(name, desc).unwrap().inline().build());
        }
        pages.push(embed.build().unwrap());
        page_count += 1;
    }
    paginate_embed(ctx, msg, pages, page_count).await?;
    Ok(())
}
