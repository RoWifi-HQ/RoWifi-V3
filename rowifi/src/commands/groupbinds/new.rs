use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::bind::GroupBind;
use twilight_mention::Mention;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct GroupbindsNewArguments {
    #[arg(help = "The Roblox Group Id to create a bind with")]
    pub group_id: i64,
    #[arg(help = "The discord roles to add to the bind", rest)]
    pub roles: String,
}

pub async fn groupbinds_new(ctx: CommandContext, args: GroupbindsNewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let group_id = args.group_id;
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
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles: Vec<i64> = Vec::new();
    for r in args.roles.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    let bind = GroupBind {
        group_id,
        discord_roles: roles,
        priority: 0,
        template: None,
    };
    let bind_bson = to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"GroupBinds": bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .unwrap()
        .description("Asset Bind Addition")
        .unwrap()
        .field(EmbedFieldBuilder::new(name, value).unwrap())
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
