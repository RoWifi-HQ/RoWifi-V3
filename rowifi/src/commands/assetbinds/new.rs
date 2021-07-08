use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::bind::{AssetBind, AssetType, Template};
use twilight_mention::Mention;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct NewArguments {
    #[arg(help = "The type of asset to create")]
    pub option: AssetType,
    #[arg(help = "The ID of asset to bind")]
    pub asset_id: i64,
    #[arg(help = "The template to be used for the bind. Can be initialized as `N/A`, `disable`")]
    pub template: String,
    #[arg(help = "The number that tells the bot which bind to choose for the nickname")]
    pub priority: Option<i64>,
    #[arg(help = "The Discord Roles to add to the bind", rest)]
    pub discord_roles: Option<String>,
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
            .color(Color::Red as u32)
            .description(format!("A bind with asset id {} already exists", asset_id))
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

    let bind = AssetBind {
        id: asset_id,
        asset_type,
        discord_roles: roles,
        priority,
        template: Some(Template(template_str.clone())),
    };
    let bind_bson = to_bson(&bind)?;

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"AssetBinds": &bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", asset_id);
    let value = format!(
        "Type: {}\nTemplate: `{}`\nPriority: {}\nRoles: {}",
        bind.asset_type,
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
    let message_id = ctx.respond()
        .embed(embed)
        .component(Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    style: ButtonStyle::Danger,
                    emoji: Some(ComponentEmoji{
                        id: None,
                        name: "🗑️".into(),
                        animated: false
                    }),
                    label: Some("Oh no! Delete?".into()),
                    custom_id: Some("ab-new-delete".into()),
                    url: None,
                    disabled: false
                })
            ]
        }))
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Asset Bind Addition")
        .field(EmbedFieldBuilder::new(name, value))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = if let Some(interaction_token) = &ctx.interaction_token {
        let msg = ctx.bot.http.get_interaction_original(interaction_token).unwrap().await?;
        if let Some(msg) = msg {
            msg.id
        } else {
            return Ok(());
        }
    } else {
        message_id.unwrap()
    };
    let author_id = ctx.author.id;

    let stream = ctx.bot.standby.wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.as_ref().member.as_ref().unwrap().user.as_ref().unwrap().id;
                if component_interaction_author == author_id {
                    let filter = doc! {"_id": guild.id};
                    let update = doc! {"$pull": {"AssetBinds": bind_bson}};
                    ctx.bot.database.modify_guild(filter, update).await?;
                    ctx.bot.http.interaction_callback(
                        message_component.id, 
                        &message_component.token, 
                        InteractionResponse::UpdateMessage(CallbackData {
                            allowed_mentions: None,
                            content: None,
                            components: Some(Vec::new()),
                            embeds: Vec::new(),
                            flags: None,
                            tts: None
                        })
                    ).await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Successful!")
                        .description("The newly created bind was deleted")
                        .build()
                        .unwrap();
                    ctx.bot.http.create_followup_message(&message_component.token).unwrap()
                        .embeds(vec![embed])
                        .await?;

                    return Ok(());
                } else {
                    let _ = ctx.bot.http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            InteractionResponse::DeferredUpdateMessage,
                        )
                        .await;
                    let _ = ctx.bot.http.create_followup_message(&message_component.token).unwrap()
                        .ephemeral(true)
                        .content("This button is only interactable by the original command invoker")
                        .await;
                }
            }
        }
    }

    if let Some(interaction_token) = &ctx.interaction_token {
        ctx.bot.http.update_interaction_original(interaction_token).unwrap().components([]).unwrap().await?;
    } else {
        ctx.bot.http.update_message(ctx.channel_id, message_id).components([]).unwrap().await?;
    }

    Ok(())
}
