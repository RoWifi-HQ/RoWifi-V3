use crate::framework::prelude::*;

use crate::models::guild::RoGuild;
use crate::utils::error::RoError;


pub static RANKBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["modify", "m"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static RANKBINDS_MODIFY_COMMAND: Command = Command {
    fun: rankbinds_modify,
    options: &RANKBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn rankbinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild = match ctx.database.get_guild(msg.guild_id.unwrap().0).await? {
        Some(g) => g,
        None => {
            return Ok(())
        }
    };

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
    };

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    let rank_id = match args.next().map(|r| r.parse::<i64>()) {
        Some(Ok(s)) => s,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    if !guild.rankbinds.iter().any(|r| r.group_id == group_id && r.rank_id == rank_id) {
        return Ok(())
    }

    if field.eq_ignore_ascii_case("prefix") {
        let prefix = match args.next() {
            Some(s) => s.to_owned(),
            None => return Ok(())
        };
        let _ = modify_prefix(ctx, &guild, group_id, rank_id, &prefix).await?;
    }

    Ok(())
}

async fn modify_prefix(ctx: &Context, guild: &RoGuild, group_id: i64, rank_id: i64, prefix: &str) -> Result<(), RoError> {
    let filter = bson::doc! {"_id": guild.id, "RankBinds.GroupId": group_id, "RankBinds.RbxRankId": rank_id};
    let update = bson::doc! {"$set": {"RankBinds.$.Prefix": prefix.clone()}};
    ctx.database.modify_guild(filter, update).await
}