use crate::framework::prelude::*;
use rowifi_models::bind::GroupBind;
use twilight_embed_builder::EmbedFieldBuilder;
use twilight_mention::Mention;

pub static GROUPBINDS_NEW_OPTIONS: CommandOptions = CommandOptions {
    perm_level: RoLevel::Admin,
    bucket: None,
    names: &["new"],
    desc: Some("Command to add a new group bind"),
    usage: Some("groupbinds new <Group Id> [Roles..]"),
    examples: &[
        "groupbinds new 3108077 @Role1",
        "gb new 5581309 @Role1 @Role2",
    ],
    min_args: 2,
    hidden: false,
    sub_commands: &[],
    group: None,
};

pub static GROUPBINDS_NEW_COMMAND: Command = Command {
    fun: groupbinds_new,
    options: &GROUPBINDS_NEW_OPTIONS,
};

#[command]
pub async fn groupbinds_new(
    ctx: &Context,
    msg: &Message,
    mut args: Arguments<'fut>,
) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild = ctx
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(RoError::Command(CommandError::NoRoGuild))?;

    let group_str = match args.next() {
        Some(g) => g,
        None => return Ok(()),
    };
    let group_id = match group_str.parse::<i64>() {
        Ok(g) => g,
        Err(_) => {
            return Err(RoError::Command(CommandError::ParseArgument(
                group_str.to_string(),
                "Group Id".into(),
                "Number".into(),
            )))
        }
    };

    if guild.groupbinds.iter().any(|g| g.group_id == group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Addition Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description(format!("A bind with group id {} already exists", group_id))
            .unwrap()
            .build()
            .unwrap();
        let _ = ctx
            .http
            .create_message(msg.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let server_roles = ctx.cache.roles(msg.guild_id.unwrap());
    let mut roles: Vec<i64> = Vec::new();
    for r in args {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    let bind = GroupBind {
        group_id,
        discord_roles: roles,
    };
    let bind_bson = bson::to_bson(&bind)?;

    let filter = bson::doc! {"_id": guild.id};
    let update = bson::doc! {"$push": {"GroupBinds": bind_bson}};
    ctx.database.modify_guild(filter, update).await?;

    let name = format!("Group: {}", group_id);
    let value = format!(
        "Roles: {}",
        bind.discord_roles
            .iter()
            .map(|r| RoleId(*r as u64).mention().to_string())
            .collect::<String>()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .unwrap()
        .color(Color::DarkGreen as u32)
        .unwrap()
        .field(EmbedFieldBuilder::new(name.clone(), value.clone()).unwrap())
        .build()
        .unwrap();
    let _ = ctx
        .http
        .create_message(msg.channel_id)
        .embed(embed)
        .unwrap()
        .await;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", msg.author.name))
        .unwrap()
        .description("Asset Bind Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, value).unwrap())
        .build()
        .unwrap();
    ctx.logger.log_guild(ctx, guild_id, log_embed).await;
    Ok(())
}
