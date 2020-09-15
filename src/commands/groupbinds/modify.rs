use crate::framework::prelude::*;
use crate::models::guild::RoGuild;

pub static GROUPBINDS_MODIFY_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["modify", "m"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[],
    group: None
};

pub static GROUPBINDS_MODIFY_COMMAND: Command = Command {
    fun: groupbinds_modify,
    options: &GROUPBINDS_MODIFY_OPTIONS
};

#[command]
pub async fn groupbinds_modify(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx.database.get_guild(guild_id.0).await?.ok_or_else(|| RoError::Command(CommandError::NoRoGuild))?;

    let field = match args.next() {
        Some(s) => s.to_owned(),
        None => return Ok(())
    };

    let group_id = match args.next().map(|g| g.parse::<i64>()) {
        Some(Ok(g)) => g,
        Some(Err(_)) => return Ok(()),
        None => return Ok(())
    };

    if !guild.groupbinds.iter().any(|g| g.group_id == group_id) {
        return Ok(())
    }

    if field.eq_ignore_ascii_case("roles-add") {
        add_roles(ctx, &guild, group_id, args).await?;
    } else if field.eq_ignore_ascii_case("roles-remove") {
        remove_roles(ctx, &guild, group_id, args).await?;
    } 

    let e = EmbedBuilder::new().default_data().color(Color::DarkGreen as u32).unwrap()
        .title("Success!").unwrap()
        .description("The bind was successfully modified").unwrap()
        .build().unwrap();

    let _ = ctx.http.create_message(msg.channel_id).embed(e).unwrap().await?;
    Ok(())
}

async fn add_roles(ctx: &Context, guild: &RoGuild, group_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "GroupBinds.GroupId": group_id};
    let update = bson::doc! {"$push": {"GroupBinds.$.DiscordRoles": {"$each": role_ids}}};
    ctx.database.modify_guild(filter, update).await
}

async fn remove_roles(ctx: &Context, guild: &RoGuild, group_id: i64, mut args: Arguments<'_>) -> Result<(), RoError> {
    let mut role_ids = Vec::new();
    while let Some(r) = args.next() {
        if let Some(r) = parse_role(r) {
            role_ids.push(r);
        }
    }
    let filter = bson::doc! {"_id": guild.id, "GroupBinds.GroupId": group_id};
    let update = bson::doc! {"$pullAll": {"GroupBinds.$.DiscordRoles": role_ids}};
    ctx.database.modify_guild(filter, update).await
}