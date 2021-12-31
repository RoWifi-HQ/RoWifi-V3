use rowifi_database::postgres::types::ToSql;
use rowifi_framework::prelude::*;
use rowifi_models::{
    guild::{GuildType, RoGuild},
    id::RoleId,
};

#[derive(FromArgs)]
pub struct FunctionalArguments {
    #[arg(help = "Discord role to edit")]
    pub role: RoleId,
}

pub async fn functional(ctx: CommandContext, args: FunctionalArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let mut guild = ctx.bot.database.get_guild(guild_id).await?;

    if guild.kind == GuildType::Free {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Command Failed")
            .description("This command is only available on Premium servers")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
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

    let message = ctx
        .respond()
        .components(&[Component::ActionRow(ActionRow {
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
                        description: Some("RoWifi will not nickname users with this role".into()),
                        emoji: None,
                        label: "RoWifi Nickname Bypass".into(),
                        value: "rowifi-nickname-bypass".into(),
                    },
                ],
            })],
        })])?
        .content("Choose permissions to give:")?
        .exec()
        .await?
        .model()
        .await?;

    let message_id = message.id;
    let author_id = ctx.author.id;
    let role_id = args.role;

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
                if component_interaction_author == author_id {
                    let mut updates = Vec::new();

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-admin")
                        && !guild.admin_roles.contains(&role_id)
                    {
                        updates.push(format!(
                            "admin_roles = array_append(admin_roles, ${})",
                            updates.len() + 1
                        ));
                        ctx.bot
                            .admin_roles
                            .entry(guild_id)
                            .or_default()
                            .push(role_id);
                    } else if guild.admin_roles.contains(&role_id) {
                        updates.push(format!(
                            "admin_roles = array_remove(admin_roles, ${})",
                            updates.len() + 1
                        ));
                        if let Some(mut admin_roles) = ctx.bot.admin_roles.get_mut(&guild_id) {
                            admin_roles.retain(|a| !a.eq(&role_id));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-trainer")
                        && !guild.trainer_roles.contains(&role_id)
                    {
                        updates.push(format!(
                            "trainer_roles = array_append(trainer_roles, ${})",
                            updates.len() + 1
                        ));
                        ctx.bot
                            .trainer_roles
                            .entry(guild_id)
                            .or_default()
                            .push(role_id);
                    } else if guild.trainer_roles.contains(&role_id) {
                        updates.push(format!(
                            "trainer_roles = array_remove(trainer_roles, ${})",
                            updates.len() + 1
                        ));
                        if let Some(mut trainer_roles) = ctx.bot.trainer_roles.get_mut(&guild_id) {
                            trainer_roles.retain(|a| !a.eq(&role_id));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-bypass")
                        && !guild.bypass_roles.contains(&role_id)
                    {
                        updates.push(format!(
                            "bypass_roles = array_append(bypass_roles, ${})",
                            updates.len() + 1
                        ));
                        ctx.bot
                            .bypass_roles
                            .entry(guild_id)
                            .or_default()
                            .push(role_id);
                    } else if guild.bypass_roles.contains(&role_id) {
                        updates.push(format!(
                            "bypass_roles = array_remove(bypass_roles, ${})",
                            updates.len() + 1
                        ));
                        if let Some(mut bypass_roles) = ctx.bot.bypass_roles.get_mut(&guild_id) {
                            bypass_roles.retain(|a| !a.eq(&role_id));
                        }
                    }

                    if message_component
                        .data
                        .values
                        .iter()
                        .any(|r| r == "rowifi-nickname-bypass")
                        && !guild.nickname_bypass_roles.contains(&role_id)
                    {
                        updates.push(format!(
                            "nickname_bypass_roles = array_append(nickname_bypass_roles, ${})",
                            updates.len() + 1
                        ));
                        ctx.bot
                            .nickname_bypass_roles
                            .entry(guild_id)
                            .or_default()
                            .push(role_id);
                    } else if guild.nickname_bypass_roles.contains(&role_id) {
                        updates.push(format!(
                            "nickname_bypass_roles = array_remove(nickname_bypass_roles, ${})",
                            updates.len() + 1
                        ));
                        if let Some(mut nickname_bypass_roles) =
                            ctx.bot.nickname_bypass_roles.get_mut(&guild_id)
                        {
                            nickname_bypass_roles.retain(|a| !a.eq(&role_id));
                        }
                    }

                    let set = updates.join(", ");
                    let mut args: Vec<&(dyn ToSql + Sync)> = vec![&role_id; updates.len()];
                    args.push(&guild.guild_id);
                    guild = ctx
                        .bot
                        .database
                        .query_one::<RoGuild>(
                            &format!(
                                "UPDATE guilds SET {} WHERE guild_id = ${} RETURNING *",
                                set,
                                updates.len() + 1
                            ),
                            &args,
                        )
                        .await?;
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
                        .exec()
                        .await;
                }
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
