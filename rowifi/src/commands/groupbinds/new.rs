use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::bind::{GroupBind, Template};
use twilight_mention::Mention;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct GroupbindsNewArguments {
    #[arg(help = "The Roblox Group Id to create a bind with")]
    pub group_id: i64,
    #[arg(help = "The template to be used for the bind")]
    pub template: String,
    #[arg(help = "The number that tells the bot which bind to choose for the nickname")]
    pub priority: Option<i64>,
    #[arg(help = "The discord roles to add to the bind", rest)]
    pub discord_roles: Option<String>,
}

pub async fn groupbinds_new(ctx: CommandContext, args: GroupbindsNewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let group_id = args.group_id;
    if guild.groupbinds.iter().any(|g| g.group_id == group_id) {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Addition Failed")
            .color(Color::Red as u32)
            .description(format!("A bind with group id {} already exists", group_id))
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let template = args.template;
    let template_str = match template.as_str() {
        "disable" => "{discord-name}".into(),
        "N/A" => "{roblox-username}".into(),
        _ => {
            if Template::has_slug(template.as_str()) {
                template.clone()
            } else {
                format!("{} {{roblox-username}}", template)
            }
        }
    };

    let priority = args.priority.unwrap_or_default();

    let discord_roles_str = args.discord_roles.unwrap_or_default();
    let roles_to_add = discord_roles_str
        .split_ascii_whitespace()
        .collect::<Vec<_>>();

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles = Vec::new();
    for r in roles_to_add {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }

    let bind = GroupBind {
        group_id,
        discord_roles: roles,
        priority,
        template: Some(Template(template_str.clone())),
    };
    let bind_bson = to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"GroupBinds": bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Group: {}", group_id);
    let value = format!(
        "Template: `{}`\nPriority: {}\nRoles: {}",
        template_str,
        priority,
        bind.discord_roles
            .iter()
            .map(|r| RoleId(*r as u64).mention().to_string())
            .collect::<String>()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .field(EmbedFieldBuilder::new(name.clone(), value.clone()))
        .build()
        .unwrap();
    ctx.respond().embed(embed).await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Asset Bind Addition")
        .field(EmbedFieldBuilder::new(name, value))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;
    Ok(())
}
