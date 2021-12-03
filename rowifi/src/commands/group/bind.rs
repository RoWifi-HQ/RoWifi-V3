use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{Assetbind, AssetType, Groupbind, Rankbind, Template, BindType},
    discord::id::{GuildId, RoleId},
    roblox::{group::PartialRank, id::GroupId},
};
use std::str::FromStr;

use crate::commands::{
    custombinds::new::custombinds_new_common, log_rankbind, new::CustombindsNewArguments,
};

lazy_static! {
    pub static ref TEMPLATE_REGEX: Regex = Regex::new(r"\[(.*?)\]").unwrap();
}

pub async fn bind(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();

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

    let message = ctx
        .respond()
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(select_menu.clone())],
        })])?
        .content("What type of bind would you like to create?")?
        .exec()
        .await?
        .model()
        .await?;

    let message_id = message.id;
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
                            &InteractionResponse::UpdateMessage(CallbackData {
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
                        .exec()
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
                        &InteractionResponse::DeferredUpdateMessage,
                    )
                    .exec()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .exec()
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
        "custom" => bind_custom(ctx, guild_id).await?,
        "asset" => bind_asset(ctx, guild_id).await?,
        "group" => bind_group(ctx, guild_id).await?,
        _ => {}
    }

    Ok(())
}

async fn bind_custom(ctx: CommandContext, guild_id: GuildId) -> CommandResult {
    let code = await_reply("Enter the code for this bind.", &ctx).await?;

    custombinds_new_common(
        ctx,
        guild_id,
        CustombindsNewArguments {
            code,
            template: None,
            priority: None,
            discord_roles: None,
        },
    )
    .await
}

async fn bind_asset(ctx: CommandContext, guild_id: GuildId) -> CommandResult {
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

    let message = ctx
        .respond()
        .content("Select the type of asset to bind:")?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(select_menu.clone())],
        })])?
        .exec()
        .await?
        .model()
        .await?;

    let message_id = message.id;
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
                    let _ = ctx
                        .bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
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
                        .exec()
                        .await;
                    asset_type = Some(message_component.data.values[0].clone());
                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        &InteractionResponse::DeferredUpdateMessage,
                    )
                    .exec()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .create_followup_message(&message_component.token)
                    .unwrap()
                    .ephemeral(true)
                    .content("This component is only interactable by the original command invoker")
                    .exec()
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
            ctx.respond().embeds(&[embed])?.exec().await?;
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
        .parse::<i32>()
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
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId::new(role_id).unwrap()) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let bind = Assetbind {
        // 0 is entered here since this field is not used in the insertion. The struct is only constructed to ensure we have
        // collected all fields.
        bind_id: 0,
        asset_id,
        asset_type,
        discord_roles: discord_roles.into_iter().unique().collect::<Vec<_>>(),
        priority,
        template: template.clone(),
    };
    
    ctx.bot.database.execute(
        "INSERT INTO binds(bind_type, guild_id, asset_id, asset_type, discord_roles, priority, template) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING bind_id",
        &[&BindType::Asset, &(guild_id.get() as i64), &bind.asset_id, &bind.asset_type, &bind.discord_roles, &bind.priority, &bind.template]
    ).await?;

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
    ctx.respond().embeds(&[embed])?.exec().await?;

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

async fn bind_group(ctx: CommandContext, guild_id: GuildId) -> CommandResult {
    let group_id = match await_reply("Enter the group id you would like to bind", &ctx)
        .await?
        .parse::<i64>()
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
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let roblox_group = match ctx.bot.roblox.get_group_ranks(GroupId(group_id as u64)).await? {
        Some(r) => r,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Bind Addition Failed")
                .description(format!("Group with Id {} does not exist", group_id))
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
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
        ctx.respond().embeds(&[embed])?.exec().await?;
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
        .parse::<i32>()
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
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId::new(role_id).unwrap()) {
                discord_roles.push(role_id as i64);
            }
        }
    }
    discord_roles = discord_roles.into_iter().unique().collect();

    let should_groupbind =
        rank_ids.len() == roblox_group.roles.len() - 1 && rank_ids.iter().any(|r| r.rank != 0);
    if !should_groupbind || template.0 == "auto" {
        return bind_rank(
            ctx,
            guild_id,
            group_id as i64,
            rank_ids,
            template,
            priority,
            discord_roles,
        )
        .await;
    }

    let bind = Groupbind {
        // 0 is entered here since this field is not used in the insertion. The struct is only constructed to ensure we have
        // collected all fields.
        bind_id: 0,
        group_id,
        discord_roles: discord_roles.into_iter().unique().collect::<Vec<_>>(),
        priority,
        template,
    };
    
    ctx.bot.database.execute(
        "INSERT INTO binds(bind_type, guild_id, group_id, discord_roles, priority, template) VALUES($1, $2, $3, $4, $5, $6) RETURNING bind_id", 
        &[&BindType::Group, &(guild_id.get() as i64), &bind.group_id, &bind.discord_roles, &bind.priority, &bind.template]
    ).await?;

    let name = format!("Group: {}", group_id);
    let value = format!(
        "Template: `{}`\nPriority: {}\nRoles: {}",
        &bind.template,
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
    ctx.respond().embeds(&[embed])?.exec().await?;

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
    guild_id: GuildId,
    group_id: i64,
    rank_ids: Vec<&PartialRank>,
    template: Template,
    priority: i32,
    discord_roles: Vec<i64>,
) -> CommandResult {
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let rankbinds = ctx.bot.database.query::<Rankbind>("SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2", &[&(guild_id.0.get() as i64), &BindType::Rank]).await?;

    let mut database = ctx.bot.database.get().await?;
    let transaction = database.transaction().await?;

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
        let bind = Rankbind {
            bind_id: 0,
            group_id,
            group_rank_id: rank_id,
            roblox_rank_id: group_rank.id.0 as i64,
            priority,
            discord_roles: discord_roles.clone(),
            template,
        };

        match rankbinds
            .iter()
            .find(|r| r.group_id == group_id && r.group_rank_id == rank_id)
        {
            Some(existing) => {
                let stmt = transaction.prepare_cached("UPDATE binds SET priority = $1, template = $2, discord_roles = $3 WHERE bind_id = $4").await?;
                transaction.execute(&stmt, &[&bind.priority, &bind.template, &bind.discord_roles, &existing.bind_id]).await?;
                modified.push(bind);
            }
            None => {
                let stmt = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, group_id, group_rank_id, roblox_rank_id, template, priority, discord_roles) VALUES($1, $2, $3, $4, $5, $6, $7, $8)").await?;
                transaction.execute(&stmt, &[&BindType::Rank, &(guild_id.get() as i64), &bind.group_id, &bind.group_rank_id, &bind.roblox_rank_id, &bind.template, &bind.priority, &bind.discord_roles]).await?;
                added.push(bind);
            }
        }
    }

    transaction.commit().await?;

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

    ctx.respond().embeds(&[embed])?.exec().await?;

    for rb in added {
        log_rankbind(&ctx, rb).await;
    }
    for rb in modified {
        log_rankbind(&ctx, rb).await;
    }

    Ok(())
}
