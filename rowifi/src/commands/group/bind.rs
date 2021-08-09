use itertools::Itertools;
use lazy_static::lazy_static;
use mongodb::bson::{self, doc};
use regex::Regex;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{AssetBind, AssetType, GroupBind, RankBind, Template},
    discord::id::{GuildId, RoleId},
    guild::RoGuild,
    roblox::{group::PartialRank, id::GroupId},
};
use std::str::FromStr;

use crate::commands::{custombinds::new::custombinds_new_common, log_rankbind, new::CustombindsNewArguments};

lazy_static! {
    pub static ref TEMPLATE_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}

pub async fn bind(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let mut select_menu = SelectMenu {
        custom_id: "bind-select".into(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options: vec![
            SelectMenuOption {
                default: false,
                description: Some("Used to bind a Roblox asset, badge or gamepass".into()),
                emoji: None,
                label: "Asset".into(),
                value: "asset".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Use our Lua-like language to create your own bind".into()),
                emoji: None,
                label: "Custom".into(),
                value: "custom".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Bind the ranks of a Roblox Group".into()),
                emoji: None,
                label: "Group".into(),
                value: "group".into(),
            },
        ],
        placeholder: None,
    };

    let message_id = ctx
        .respond()
        .component(Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(select_menu.clone())],
        }))
        .content("What type of bind would you like to create?")
        .await?;

    let message_id = message_id.unwrap();
    let author_id = ctx.author.id;
    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    select_menu.disabled = true;

    ctx.bot.ignore_message_components.insert(message_id);
    let mut bind_type = None;
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                components: Some(vec![Component::ActionRow(ActionRow {
                                    components: vec![Component::SelectMenu(select_menu.clone())],
                                })]),
                                content: None,
                                embeds: Vec::new(),
                                flags: None,
                                tts: Some(false),
                            }),
                        )
                        .await?;
                    bind_type = Some(message_component.data.values[0].clone());
                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        InteractionResponse::DeferredUpdateMessage,
                    )
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    let bind_type = match bind_type {
        Some(b) => b,
        None => return Ok(()),
    };

    match bind_type.as_str() {
        "custom" => bind_custom(ctx, guild_id, guild).await?,
        "asset" => bind_asset(ctx, guild_id, guild).await?,
        "group" => bind_group(ctx, guild_id, guild).await?,
        _ => {}
    }

    Ok(())
}

async fn bind_custom(ctx: CommandContext, guild_id: GuildId, guild: RoGuild) -> CommandResult {
    let code = await_reply("Enter the code for this bind.", &ctx).await?;

    custombinds_new_common(ctx, guild_id, guild, CustombindsNewArguments {
        code,
        template: None,
        priority: None,
        discord_roles: None
    }).await
}

async fn bind_asset(ctx: CommandContext, guild_id: GuildId, guild: RoGuild) -> CommandResult {
    let mut select_menu = SelectMenu {
        custom_id: "bind-select-asset".into(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options: vec![
            SelectMenuOption {
                default: false,
                description: None,
                emoji: None,
                label: "Asset".into(),
                value: "asset".into(),
            },
            SelectMenuOption {
                default: false,
                description: None,
                emoji: None,
                label: "Badge".into(),
                value: "badge".into(),
            },
            SelectMenuOption {
                default: false,
                description: None,
                emoji: None,
                label: "Gamepass".into(),
                value: "gamepass".into(),
            },
        ],
        placeholder: None,
    };

    let message_id = ctx
        .respond()
        .content("Select the type of asset to bind:")
        .components(vec![Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(select_menu.clone())],
        })])
        .await?;

    let message_id = message_id.unwrap();
    let author_id = ctx.author.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    select_menu.disabled = true;

    ctx.bot.ignore_message_components.insert(message_id);
    let mut asset_type = None;
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    let _ = ctx.bot.http.interaction_callback(
                        message_component.id,
                        &message_component.token,
                        InteractionResponse::UpdateMessage(CallbackData {
                            allowed_mentions: None,
                            components: Some(vec![Component::ActionRow(ActionRow {
                                components: vec![Component::SelectMenu(select_menu.clone())],
                            })]),
                            content: None,
                            embeds: Vec::new(),
                            flags: None,
                            tts: Some(false),
                        }),
                    );
                    asset_type = Some(message_component.data.values[0].clone());
                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        InteractionResponse::DeferredUpdateMessage,
                    )
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    let asset_type = match asset_type {
        Some(s) => match AssetType::from_arg(&s) {
            Ok(a) => a,
            Err(_) => return Ok(()),
        },
        None => return Ok(()),
    };

    let asset_id = match await_reply("Enter the id of the asset to bind.", &ctx)
        .await?
        .parse::<i64>()
    {
        Ok(a) => a,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Asset Bind Addition Failed")
                .description("Expected asset id to be a number")
                .build()
                .unwrap();
            ctx.respond().embeds(vec![embed]).await?;
            return Ok(());
        }
    };

    let select_menu = SelectMenu {
        custom_id: "template-reply".into(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options: vec![
            SelectMenuOption {
                default: true,
                description: Some("Sets the nickname as just the roblox username".into()),
                emoji: None,
                label: "{roblox-username}".into(),
                value: "{roblox-username}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the roblox id of the user".into()),
                emoji: None,
                label: "{roblox-id}".into(),
                value: "{roblox-id}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the discord id of the user".into()),
                emoji: None,
                label: "{discord-id}".into(),
                value: "{discord-id}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the discord username".into()),
                emoji: None,
                label: "{discord-name}".into(),
                value: "{discord-name}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the display name on Roblox".into()),
                emoji: None,
                label: "{display-name}".into(),
                value: "{display-name}".into(),
            },
        ],
        placeholder: None,
    };
    let template = await_template_reply(
        "Enter the template you wish to set for the bind.\nSelect one of the below or enter your own.",
        &ctx,
        select_menu
    )
    .await?;

    let priority = match await_reply("Enter the priority you wish to set for the bind.", &ctx)
        .await?
        .parse::<i64>()
    {
        Ok(p) => p,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Bind Addition Failed")
                .description("Expected priority to be a number")
                .build()
                .unwrap();
            ctx.respond().embeds(vec![embed]).await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId(role_id)) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let bind = AssetBind {
        id: asset_id,
        asset_type,
        discord_roles,
        priority,
        template: Some(template.clone()),
    };
    let bind_bson = bson::to_bson(&bind)?;

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"AssetBinds": &bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", asset_id);
    let value = format!(
        "Type: {}\nTemplate: `{}`\nPriority: {}\nRoles: {}",
        bind.asset_type,
        template.0,
        priority,
        bind.discord_roles
            .iter()
            .map(|r| format!("<@&{}> ", r))
            .collect::<String>()
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .field(EmbedFieldBuilder::new(name.clone(), value.clone()))
        .build()
        .unwrap();
    ctx.respond().embeds(vec![embed]).await?;

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

async fn bind_group(ctx: CommandContext, guild_id: GuildId, guild: RoGuild) -> CommandResult {
    let group_id = match await_reply("Enter the group id you would like to bind", &ctx)
        .await?
        .parse::<u64>()
    {
        Ok(p) => p,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Bind Addition Failed")
                .description("Expected the group id to be a number")
                .build()
                .unwrap();
            ctx.respond().embeds(vec![embed]).await?;
            return Ok(());
        }
    };

    let roblox_group = match ctx.bot.roblox.get_group_ranks(GroupId(group_id)).await? {
        Some(r) => r,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Bind Addition Failed")
                .description(format!("Group with Id {} does not exist", group_id))
                .build()
                .unwrap();
            ctx.respond().embeds(vec![embed]).await?;
            return Ok(());
        }
    };

    let rank_ids_str = await_reply("Enter the rank ids you would like to bind.\nYou may enter these as a range or space separated numbers. Enter all if you would like to bind all ranks", &ctx).await?;

    let mut rank_ids = Vec::new();
    for id in rank_ids_str.split_ascii_whitespace() {
        if id.eq_ignore_ascii_case("all") {
            rank_ids = roblox_group.roles.iter().filter(|r| r.rank != 0).collect();
            break;
        }
        if let Ok(r) = RankId::from_str(id) {
            match r {
                RankId::Range(r1, r2) => {
                    let ids = roblox_group
                        .roles
                        .iter()
                        .filter(|r| i64::from(r.rank) >= r1 && i64::from(r.rank) <= r2);
                    rank_ids.extend(ids);
                }
                RankId::Single(r) => {
                    if let Some(rank) = roblox_group.roles.iter().find(|gr| i64::from(gr.rank) == r)
                    {
                        rank_ids.push(rank);
                    }
                }
            }
        }
    }

    if rank_ids.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Bind Addition Failed")
            .description("There were no ranks found associated with the entered ones")
            .build()
            .unwrap();
        ctx.respond().embeds(vec![embed]).await?;
        return Ok(());
    }

    let rank_ids = rank_ids.into_iter().unique().collect::<Vec<_>>();

    let select_menu = SelectMenu {
        custom_id: "template-reply".into(),
        disabled: false,
        max_values: Some(1),
        min_values: Some(1),
        options: vec![
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as just the roblox username".into()),
                emoji: None,
                label: "{roblox-username}".into(),
                value: "{roblox-username}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the roblox id of the user".into()),
                emoji: None,
                label: "{roblox-id}".into(),
                value: "{roblox-id}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the discord id of the user".into()),
                emoji: None,
                label: "{discord-id}".into(),
                value: "{discord-id}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the discord username".into()),
                emoji: None,
                label: "{discord-name}".into(),
                value: "{discord-name}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Sets the nickname as the display name on Roblox".into()),
                emoji: None,
                label: "{display-name}".into(),
                value: "{display-name}".into(),
            },
            SelectMenuOption {
                default: false,
                description: Some("Let RoWifi determine the template from the ranks".into()),
                emoji: None,
                label: "auto".into(),
                value: "auto".into(),
            },
        ],
        placeholder: None,
    };

    let template = await_template_reply(
        "Enter the template you wish to set for the bind.\nSelect one of the below or enter your own.",
        &ctx,
        select_menu
    )
    .await?;

    let priority = match await_reply("Enter the priority you wish to set for the bind.", &ctx)
        .await?
        .parse::<i64>()
    {
        Ok(p) => p,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Bind Addition Failed")
                .description("Expected priority to be a number")
                .build()
                .unwrap();
            ctx.respond().embeds(vec![embed]).await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId(role_id)) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let should_groupbind =
        rank_ids.len() == roblox_group.roles.len() - 1 && rank_ids.iter().any(|r| r.rank != 0);
    if !should_groupbind || template.0 == "auto" {
        return bind_rank(
            ctx,
            guild,
            group_id as i64,
            rank_ids,
            template,
            priority,
            discord_roles,
        )
        .await;
    }

    let bind = GroupBind {
        group_id: group_id as i64,
        discord_roles,
        priority,
        template: Some(template.clone()),
    };

    let bind_bson = bson::to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"GroupBinds": &bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Group: {}", group_id);
    let value = format!(
        "Template: `{}`\nPriority: {}\nRoles: {}",
        &template.0,
        priority,
        bind.discord_roles
            .iter()
            .map(|r| format!("<@&{}> ", r))
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
        .description("Bind Addition")
        .field(EmbedFieldBuilder::new(name, value))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    Ok(())
}

async fn bind_rank(
    ctx: CommandContext,
    mut guild: RoGuild,
    group_id: i64,
    rank_ids: Vec<&PartialRank>,
    template: Template,
    priority: i64,
    discord_roles: Vec<i64>,
) -> CommandResult {
    let mut added = Vec::new();
    let mut modified = Vec::new();

    for group_rank in rank_ids {
        let template = if template.0 == "auto" {
            let t = match TEMPLATE_REGEX.captures(&group_rank.name) {
                Some(m) => format!("[{}] {{roblox-username}}", m.get(1).unwrap().as_str()),
                None => "{roblox-username}".into(),
            };
            Template(t)
        } else {
            template.clone()
        };

        let rank_id = i64::from(group_rank.rank);
        let bind = RankBind {
            group_id,
            rank_id,
            rbx_rank_id: group_rank.id.0 as i64,
            prefix: None,
            priority,
            discord_roles: discord_roles.clone(),
            template: Some(template),
        };

        match guild
            .rankbinds
            .iter()
            .find_position(|r| r.group_id == group_id as i64 && r.rank_id == rank_id)
        {
            Some((pos, _)) => {
                guild.rankbinds[pos] = bind.clone();
                modified.push(bind);
            }
            None => {
                guild.rankbinds.push(bind.clone());
                added.push(bind);
            }
        }
    }
    ctx.bot.database.add_guild(&guild, true).await?;
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Binds Addition Sucessful")
        .color(Color::DarkGreen as u32)
        .description(format!(
            "Added {} rankbinds and modified {} rankbinds",
            added.len(),
            modified.len()
        ))
        .build()
        .unwrap();

    ctx.respond().embed(embed).await?;

    for rb in added {
        log_rankbind(&ctx, rb).await;
    }
    for rb in modified {
        log_rankbind(&ctx, rb).await;
    }

    Ok(())
}
