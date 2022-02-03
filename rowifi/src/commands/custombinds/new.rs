use itertools::Itertools;
use rowifi_database::postgres::Row;
use rowifi_framework::{constants::EMBED_DESCRIPTION_LIMIT, prelude::*};
use rowifi_models::{
    bind::{BindType, Custombind, Template},
    id::{BindId, GuildId, RoleId, UserId},
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

#[allow(clippy::option_option)]
pub struct CustombindsNewArguments {
    pub code: String,
    pub template: Option<String>,
    pub priority: Option<Option<i32>>,
    pub discord_roles: Option<Option<String>>,
}

impl FromArgs for CustombindsNewArguments {
    fn from_args(args: &mut Arguments) -> Result<Self, ArgumentError> {
        let code = match args.rest().map(|s| String::from_arg(s.as_str())) {
            Some(Ok(s)) => s,
            Some(Err(err)) => {
                return Err(ArgumentError::ParseError {
                    expected: err.0,
                    usage: Self::generate_help(),
                    name: "code",
                })
            }
            None => {
                return Err(ArgumentError::MissingArgument {
                    usage: Self::generate_help(),
                    name: "code",
                })
            }
        };

        Ok(Self {
            code,
            template: None,
            priority: None,
            discord_roles: None,
        })
    }

    fn from_interaction(options: &[CommandDataOption]) -> Result<Self, ArgumentError> {
        let options = options
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect::<std::collections::HashMap<&str, &CommandDataOption>>();

        let code = match options.get(&"code").map(|s| String::from_interaction(*s)) {
            Some(Ok(s)) => s,
            Some(Err(err)) => {
                return Err(ArgumentError::ParseError {
                    expected: err.0,
                    usage: Self::generate_help(),
                    name: "code",
                })
            }
            None => {
                return Err(ArgumentError::MissingArgument {
                    usage: Self::generate_help(),
                    name: "code",
                })
            }
        };

        let template = match options
            .get(&"template")
            .map(|s| String::from_interaction(*s))
        {
            Some(Ok(s)) => s,
            Some(Err(err)) => {
                return Err(ArgumentError::ParseError {
                    expected: err.0,
                    usage: Self::generate_help(),
                    name: "template",
                })
            }
            None => {
                return Err(ArgumentError::MissingArgument {
                    usage: Self::generate_help(),
                    name: "template",
                })
            }
        };

        let priority = match options.get(&"priority").map(|s| i32::from_interaction(*s)) {
            Some(Ok(s)) => Some(Some(s)),
            Some(Err(err)) => {
                return Err(ArgumentError::ParseError {
                    expected: err.0,
                    usage: Self::generate_help(),
                    name: "priority",
                })
            }
            None => Some(None),
        };

        let discord_roles = match options
            .get(&"discord_roles")
            .map(|s| String::from_interaction(*s))
        {
            Some(Ok(s)) => Some(Some(s)),
            Some(Err(err)) => {
                return Err(ArgumentError::ParseError {
                    expected: err.0,
                    usage: Self::generate_help(),
                    name: "discord_roles",
                })
            }
            None => Some(None),
        };

        Ok(Self {
            code,
            template: Some(template),
            priority,
            discord_roles,
        })
    }

    fn generate_help() -> (&'static str, &'static str) {
        ("code", "Code that makes up the bind")
    }
}

pub async fn custombinds_new(ctx: CommandContext, args: CustombindsNewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    custombinds_new_common(ctx, guild_id, args).await
}

pub async fn custombinds_new_common(
    ctx: CommandContext,
    guild_id: GuildId,
    args: CustombindsNewArguments,
) -> CommandResult {
    let user = match ctx
        .bot
        .database
        .get_linked_user(UserId(ctx.author.id), guild_id)
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Addition Failed")
                .description("You must be verified to create a custom bind")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx
        .member(ctx.guild_id.unwrap(), UserId(ctx.author.id))
        .await?
        .unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id, false).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(&args.code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(&s)?.exec().await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(&res)?.exec().await?;
        return Ok(());
    }

    let template = match args.template {
        Some(t) => Template(t),
        None => {
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
                ],
                placeholder: None,
            };
            await_template_reply(
                "Enter the template you wish to set for the bind.\nSelect one of the below or enter your own.",
                &ctx,
                select_menu
            )
            .await?
        }
    };

    let priority = match args.priority {
        Some(p) => p.unwrap_or_default(),
        None => {
            match await_reply("Enter the priority you wish to set for the bind.", &ctx)
                .await?
                .parse::<i32>()
            {
                Ok(p) => p,
                Err(_) => {
                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::Red as u32)
                        .title("Custom Bind Addition Failed")
                        .description("Expected priority to be a number")
                        .build()
                        .unwrap();
                    ctx.respond().embeds(&[embed])?.exec().await?;
                    return Ok(());
                }
            }
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = match args.discord_roles {
        Some(s) => s.unwrap_or_default(),
        None => await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?
    };
    let mut roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(resolved) = &ctx.resolved {
            roles.extend(resolved.roles.iter().map(|r| RoleId(*r.0)));
        } else if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&role_id) {
                roles.push(role_id);
            }
        }
    }

    let bind = Custombind {
        // default is entered here since this field is not used in inserting the bind. The struct is only created for thr purpose
        // of ensuring all fields are collected.
        bind_id: BindId::default(),
        custom_bind_id: 0,
        code: args.code.clone(),
        priority,
        command,
        discord_roles: roles.into_iter().unique().collect::<Vec<_>>(),
        template,
    };

    let row = ctx.bot.database.query_one::<Row>(r#"
        INSERT INTO binds(bind_type, guild_id, custom_bind_id, discord_roles, code, priority, template) 
        VALUES($1, $2, (SELECT COALESCE(max(custom_bind_id) + 1, 1) FROM binds WHERE guild_id = $2 AND bind_type = $1), $3, $4, $5, $6)
        RETURNING custom_bind_id, bind_id"#,
     &[&BindType::Custom, &(guild_id), &bind.discord_roles, &bind.code, &bind.priority, &bind.template]
    ).await?;
    let bind_id: BindId = row.get("bind_id");

    let mut desc = format!("**Id**\n: {}", row.get::<'_, _, i32>("custom_bind_id"));
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    desc.push_str(&format!(
        "Code: {}\nTemplate: `{}`\nPriority: {}\nDiscord Roles: {}",
        bind.code, bind.template, bind.priority, roles_str
    ));
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .description(
            desc.chars()
                .take(EMBED_DESCRIPTION_LIMIT)
                .collect::<String>(),
        )
        .build()
        .unwrap();
    let message = ctx
        .respond()
        .embeds(&[embed])?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "🗑️".into()
                }),
                label: Some("Oh no! Delete?".into()),
                custom_id: Some("cb-new-delete".into()),
                url: None,
                disabled: false,
            })],
        })])?
        .exec()
        .await?
        .model()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Custom Bind Addition")
        .description(
            desc.chars()
                .take(EMBED_DESCRIPTION_LIMIT)
                .collect::<String>(),
        )
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let author_id = ctx.author.id;
    let message_id = message.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(60));
    tokio::pin!(stream);

    ctx.bot.ignore_message_components.insert(message_id);
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    ctx.bot
                        .http
                        .interaction(ctx.bot.application_id)
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: None,
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
                        .await?;

                    ctx.bot
                        .database
                        .execute("DELETE FROM binds WHERE bind_id = $1", &[&bind_id])
                        .await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Successful!")
                        .description("The newly created bind was deleted")
                        .build()
                        .unwrap();
                    ctx.bot
                        .http
                        .interaction(ctx.bot.application_id)
                        .create_followup_message(&message_component.token)
                        .embeds(&[embed])?
                        .exec()
                        .await?;

                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction(ctx.bot.application_id)
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
                    .interaction(ctx.bot.application_id)
                    .create_followup_message(&message_component.token)
                    .ephemeral(true)
                    .content("This button is only interactable by the original command invoker")?
                    .exec()
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
