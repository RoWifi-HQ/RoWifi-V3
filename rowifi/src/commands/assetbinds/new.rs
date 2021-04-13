use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::bind::{AssetBind, AssetType};
use twilight_mention::Mention;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct NewArguments {
    #[arg(help = "The type of asset to create")]
    pub option: AssetType,
    #[arg(help = "The ID of asset to bind")]
    pub asset_id: i64,
    #[arg(help = "The Discord Roles to add to the bind", rest)]
    pub discord_roles: String,
}

pub async fn assetbinds_new(ctx: CommandContext, args: NewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx
        .bot
        .database
        .get_guild(guild_id.0)
        .await?
        .ok_or(CommonError::UnknownGuild)?;

    let asset_type = args.option;
    let asset_id = args.asset_id;
    if guild
        .assetbinds
        .iter()
        .any(|a| a.asset_type == asset_type && a.id == asset_id)
    {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Addition Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description(format!("A bind with asset id {} already exists", asset_id))
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let server_roles = ctx.bot.cache.roles(guild_id);
    let mut roles: Vec<i64> = Vec::new();
    for r in args.discord_roles.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(r) {
            if server_roles.contains(&RoleId(role_id)) {
                roles.push(role_id as i64);
            }
        }
    }
    if roles.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .title("Bind Addition Failed")
            .unwrap()
            .color(Color::Red as u32)
            .unwrap()
            .description("Atleast role must be entered to create an assetbind")
            .unwrap()
            .build()
            .unwrap();
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .embed(embed)
            .unwrap()
            .await?;
        return Ok(());
    }

    let bind = AssetBind {
        id: asset_id,
        asset_type,
        discord_roles: roles,
        priority: 0,
        template: None,
    };
    let bind_bson = to_bson(&bind)?;

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"AssetBinds": bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", asset_id);
    let value = format!(
        "Type: {}\nRoles: {}",
        bind.asset_type,
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
    ctx.bot
        .http
        .create_message(ctx.channel_id)
        .embed(embed)
        .unwrap()
        .await?;

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
