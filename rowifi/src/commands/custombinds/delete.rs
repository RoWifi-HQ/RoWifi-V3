use rowifi_database::dynamic_args;
use rowifi_framework::prelude::*;
use rowifi_models::bind::{BindType, Custombind};

#[derive(FromArgs)]
pub struct CustombindsDeleteArguments {
    #[arg(help = "The ID of the custombind to delete", rest)]
    pub id: String,
}

pub async fn custombinds_delete(
    ctx: CommandContext,
    args: CustombindsDeleteArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let custombinds = ctx
        .bot
        .database
        .query::<Custombind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type  = $2 ORDER BY custom_bind_id",
            &[&(guild_id), &BindType::Custom],
        )
        .await?;

    let mut ids_to_delete = Vec::new();
    for arg in args.id.split_ascii_whitespace() {
        if let Ok(r) = arg.parse::<i32>() {
            ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for id in ids_to_delete {
        if let Some(bind) = custombinds.iter().find(|r| r.custom_bind_id == id) {
            binds_to_delete.push(bind);
        }
    }
    let bind_ids = binds_to_delete
        .iter()
        .map(|c| c.bind_id)
        .collect::<Vec<_>>();

    if binds_to_delete.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Binds Deletion Failed")
            .description("There were no binds found associated with given ids")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }

    let db = ctx.bot.database.get().await?;
    let stmt = db
        .prepare_cached(&format!(
            "DELETE FROM binds WHERE bind_id IN ({})",
            dynamic_args(bind_ids.len())
        ))
        .await?;
    db.execute_raw(&stmt, bind_ids).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Success!")
        .description("The given binds were successfully deleted")
        .build()
        .unwrap();
    let message = ctx
        .respond()
        .embeds(&[embed])?
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "↩️".into()
                }),
                label: Some("Uh oh? Revert".into()),
                custom_id: Some("cb-delete-revert".into()),
                url: None,
                disabled: false,
            })],
        })])?
        .exec()
        .await?
        .model()
        .await?;

    let ids_str = binds_to_delete
        .iter()
        .map(|b| format!("`Id`: {}\n", b.custom_bind_id))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Custom Bind Deletion")
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = message.id;
    let author_id = ctx.author.id;

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

                    let mut db = ctx.bot.database.get().await?;
                    let transaction = db.transaction().await?;
                    let statement = transaction.prepare_cached(r#"
                        INSERT INTO binds(bind_type, guild_id, custom_bind_id, discord_roles, code, priority, template) 
                        VALUES($1, $2, (SELECT COALESCE(max(custom_bind_id) + 1, 1) FROM binds WHERE guild_id = $2 AND bind_type = $1), $3, $4, $5, $6)
                    "#).await?;
                    for bind in binds_to_delete {
                        transaction
                            .execute(
                                &statement,
                                &[
                                    &BindType::Custom,
                                    &(guild_id),
                                    &bind.discord_roles,
                                    &bind.code,
                                    &bind.priority,
                                    &bind.template,
                                ],
                            )
                            .await?;
                    }
                    transaction.commit().await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Restoration Successful!")
                        .description("The deleted binds were successfully restored")
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
