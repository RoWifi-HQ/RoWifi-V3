use crate::framework::prelude::*;
use crate::models::bind::GroupBind;
use twilight_mention::Mention;
use twilight_embed_builder::EmbedFieldBuilder;

pub static GROUPBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    allowed_roles: &[],
    bucket: None,
    names: &["new"],
    desc: None,
    usage: None,
    examples: &[],
    required_permissions: Permissions::empty(),
    hidden: false,
    owners_only: false,
    sub_commands: &[]
};

pub static GROUPBINDS_NEW_COMMAND: Command = Command {
    fun: groupbinds_new,
    options: &GROUPBINDS_NEW_OPTIONS
};

#[command]
pub async fn groupbinds_new(ctx: &Context, msg: &Message, mut args: Arguments<'fut>) -> CommandResult {
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

    if guild.groupbinds.iter().find(|g| g.group_id == group_id).is_some() {
        let embed = EmbedBuilder::new().default_data().title("Bind Addition Failed").unwrap()
            .color(Color::Red as u32).unwrap()
            .description(format!("A bind with group id {} already exists", group_id)).unwrap()
            .build().unwrap();
        let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await?;
    }

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let mut roles: Vec<i64> = Vec::new();
    while let Some(r) = args.next() {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    let bind = GroupBind {group_id, discord_roles: roles};
    let bind_bson = bson::to_bson(&bind)?;

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"GroupBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Group: {}", group_id);
    let value = format!("Roles: {}", bind.discord_roles.iter().map(|r| RoleId(*r as u64).mention().to_string()).collect::<String>());
    let embed = EmbedBuilder::new().default_data().title("Bind Addition Successful").unwrap()
        .color(Color::DarkGreen as u32).unwrap()
        .field(EmbedFieldBuilder::new(name, value).unwrap())
        .build().unwrap();
    let _ = ctx.http.create_message(msg.channel_id).embed(embed).unwrap().await;
    Ok(())
}