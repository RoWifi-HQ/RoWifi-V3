use crate::framework::prelude::*;
use crate::utils::error::RoError;

pub static RANKBINDS_DELETE_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["delete", "d"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static RANKBINDS_DELETE_COMMAND: Command = Command {
    fun: rankbinds_delete,
    options: &RANKBINDS_DELETE_OPTIONS
};

#[command]
pub async fn rankbinds_delete(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = match ctx.database.get_guild(guild_id.0).await? {
        Some(g) => g,
        None => return Err(RoError::NoRoGuild)
    };

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let mut rank_ids_to_delete = Vec::new();
    while let Some(arg) = args.next() {
        if let Ok(r) = arg.parse::<i64>() {
            rank_ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for rank in rank_ids_to_delete {
        if let Some(b) = guild.rankbinds.iter().find(|r| r.group_id == group_id && r.rank_id == rank) {
            binds_to_delete.push(b.rbx_rank_id);
        }
    }

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$pull": {"RankBinds": {"RbxGrpRoleId": {"$in": binds_to_delete}}}};
    let _ = ctx.database.modify_guild(filter, update).await?;

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The given bind were successfully deleted").unwrap()
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}