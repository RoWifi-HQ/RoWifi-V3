use mongodb::bson::doc;
use rowifi_framework::prelude::*;
use rowifi_models::guild::GuildType;
use twilight_model::id::RoleId;

#[derive(FromArgs)]
pub struct FunctionalArguments {
    #[arg(help = "Discord role to edit")]
    pub role: RoleId,
}

pub async fn functional(ctx: CommandContext, args: FunctionalArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    if guild.settings.guild_type == GuildType::Normal {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embed(embed).await?;
        return Ok(());
    }

    let is_admin = ctx
        .bot
        .admin_roles
        .get(&guild_id)
        .map(|r| r.contains(&args.role))
        .unwrap_or_default();
    let is_trainer = ctx
        .bot
        .trainer_roles
        .get(&guild_id)
        .map(|r| r.contains(&args.role))
        .unwrap_or_default();
    let is_bypass = ctx
        .bot
        .bypass_roles
        .get(&guild_id)
        .map(|r| r.contains(&args.role))
        .unwrap_or_default();
    let is_nick_bypass = ctx
        .bot
        .nickname_bypass_roles
        .get(&guild_id)
        .map(|r| r.contains(&args.role))
        .unwrap_or_default();

    let message_id = ctx
        .respond()
        .component(Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: "functional-select".into(),
                disabled: false,
                max_values: Some(4),
                min_values: Some(0),
                placeholder: Some("No permissions granted".into()),
                options: vec![
                    SelectMenuOption {
                        default: is_admin,
                        description: Some("Allows users to manage RoWifi".into()),
                        emoji: None,
                        label: "RoWifi Admin".into(),
                        value: "rowifi-admin".into(),
                    },
                    SelectMenuOption {
                        default: is_trainer,
                        description: Some("Allows users to use trainer commands".into()),
                        emoji: None,
                        label: "RoWifi Trainer".into(),
                        value: "rowifi-trainer".into(),
                    },
                    SelectMenuOption {
                        default: is_bypass,
                        description: Some("RoWifi will not update members with this role".into()),
                        emoji: None,
                        label: "RoWifi Bypass".into(),
                        value: "rowifi-bypass".into(),
                    },
                    SelectMenuOption {
                        default: is_nick_bypass,
                        description: Some(
                            "RoWifi will not nicknames of users with this role".into(),
                        ),
                        emoji: None,
                        label: "RoWifi Nickname Bypass".into(),
                        value: "rowifi-nickname-bypass".into(),
                    },
                ],
            })],
        }))
        .content("Choose permissions to give:")
        .await?;

    let message_id = message_id.unwrap();
    let author_id = ctx.author.id;
    let role_id = args.role.0 as i64;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component
                    .as_ref()
                    .member
                    .as_ref()
                    .unwrap()
                    .user
                    .as_ref()
                    .unwrap()
                    .id;
                let _ = ctx
                    .bot
                    .http
                    .interaction_callback(
                        message_component.id,
                        &message_component.token,
                        InteractionResponse::DeferredUpdateMessage,
                    )
                    .await;
                if component_interaction_author == author_id {
                    let filter = doc! {"_id": guild.id};
                    let mut updates = Vec::new();

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-admin")
                    {
                        updates.push(doc! {"$push": {"Settings.AdminRoles": role_id}});
                    } else if guild.settings.admin_roles.contains(&role_id) {
                        updates.push(doc! {"$pull": {"Settings.AdminRoles": role_id}});
                        if let Some(mut admin_roles) = ctx.bot.admin_roles.get_mut(&guild_id) {
                            admin_roles.retain(|a| !a.eq(&args.role));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-trainer")
                    {
                        updates.push(doc! {"$push": {"Settings.TrainerRoles": role_id}});
                    } else if guild.settings.trainer_roles.contains(&role_id) {
                        updates.push(doc! {"$pull": {"Settings.TrainerRoles": role_id}});
                        if let Some(mut trainer_roles) = ctx.bot.trainer_roles.get_mut(&guild_id) {
                            trainer_roles.retain(|a| !a.eq(&args.role));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-bypass")
                    {
                        updates.push(doc! {"$push": {"Settings.BypassRoles": role_id}});
                    } else if guild.settings.bypass_roles.contains(&role_id) {
                        updates.push(doc! {"$pull": {"Settings.BypassRoles": role_id}});
                        if let Some(mut bypass_roles) = ctx.bot.bypass_roles.get_mut(&guild_id) {
                            bypass_roles.retain(|a| !a.eq(&args.role));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-nickname-bypass")
                    {
                        updates.push(doc! {"$push": {"Settings.NicknameBypassRoles": role_id}});
                    } else if guild.settings.nickname_bypass_roles.contains(&role_id) {
                        updates.push(doc! {"$pull": {"Settings.NicknameBypassRoles": role_id}});
                        if let Some(mut nickname_bypass_roles) =
                            ctx.bot.nickname_bypass_roles.get_mut(&guild_id)
                        {
                            nickname_bypass_roles.retain(|a| !a.eq(&args.role));
                        }
                    }

                    for update in updates {
                        ctx.bot
                            .database
                            .modify_guild(filter.clone(), update)
                            .await?;
                    }
                } else {
                    let _ = ctx
                        .bot
                        .http
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .ephemeral(true)
                        .content(
                            "This component is only interactable by the original command invoker",
                        )
                        .await;
                }
            }
        }
    }

    if let Some(interaction_token) = &ctx.interaction_token {
        ctx.bot
            .http
            .update_interaction_original(interaction_token)
            .unwrap()
            .components(None)
            .unwrap()
            .await?;
    } else {
        ctx.bot
            .http
            .update_message(ctx.channel_id, message_id)
            .components(None)
            .unwrap()
            .await?;
    }

    Ok(())
}
