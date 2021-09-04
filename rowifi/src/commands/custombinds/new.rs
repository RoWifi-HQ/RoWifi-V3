use itertools::Itertools;
use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{CustomBind, Template},
    discord::id::{GuildId, RoleId},
    guild::RoGuild,
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

#[allow(clippy::option_option)]
pub struct CustombindsNewArguments {
    pub code: String,
    pub template: Option<String>,
    pub priority: Option<Option<i64>>,
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
            .map(|c| (c.name(), c))
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

        let priority = match options.get(&"priority").map(|s| i64::from_interaction(*s)) {
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
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    custombinds_new_common(ctx, guild_id, guild, args).await
}

pub async fn custombinds_new_common(
    ctx: CommandContext,
    guild_id: GuildId,
    guild: RoGuild,
    args: CustombindsNewArguments,
) -> CommandResult {
    let user = match ctx.get_linked_user(ctx.author.id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Addition Failed")
                .description("You must be verified to create a custom bind")
                .build()
                .unwrap();
            ctx.respond().embeds(&[embed]).exec().await?;
            return Ok(());
        }
    };
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx
        .member(ctx.guild_id.unwrap(), ctx.author.id.0)
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
            ctx.respond().content(&s).exec().await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(&res).exec().await?;
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
                .parse::<i64>()
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
                    ctx.respond().embeds(&[embed]).exec().await?;
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
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId(role_id)) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let mut binds = guild.custombinds.iter().map(|c| c.id).collect_vec();
    binds.sort_unstable();
    let id = binds.last().unwrap_or(&0) + 1;
    let bind = CustomBind {
        id,
        code: args.code.clone(),
        prefix: None,
        priority,
        command,
        discord_roles,
        template: Some(template),
    };
    let bind_bson = to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"CustomBinds": &bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", bind.id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Code: {}\nTemplate: `{}`\nPriority: {}\nDiscord Roles: {}",
        bind.code,
        bind.template.unwrap(),
        bind.priority,
        roles_str
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    let message = ctx
        .respond()
        .embeds(&[embed])
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
        })])
        .exec()
        .await?
        .model()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Custom Bind Addition")
        .field(EmbedFieldBuilder::new(name, desc))
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
                    let filter = doc! {"_id": guild.id};
                    let update = doc! {"$pull": {"CustomBinds": bind_bson}};
                    ctx.bot.database.modify_guild(filter, update).await?;
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .exec()
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
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .embeds(&[embed])
                        .exec()
                        .await?;

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
                    .content("This button is only interactable by the original command invoker")
                    .exec()
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
