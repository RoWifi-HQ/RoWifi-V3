use mongodb::bson::{self, doc};
use rowifi_framework::prelude::*;
use rowifi_models::discord::{application::interaction::Interaction, gateway::event::Event};
use std::time::Duration;
use tokio_stream::StreamExt;

#[derive(FromArgs)]
pub struct DeleteArguments {
    #[arg(help = "The ID of the Asset to delete", rest)]
    pub asset_id: String,
}

pub async fn assetbinds_delete(ctx: CommandContext, args: DeleteArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let mut assets_to_delete = Vec::new();
    for arg in args.asset_id.split_ascii_whitespace() {
        if let Ok(r) = arg.parse::<i64>() {
            assets_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for asset in assets_to_delete {
        if let Some(b) = guild.assetbinds.iter().find(|r| r.id == asset) {
            binds_to_delete.push(b);
        }
    }
    let bind_ids = binds_to_delete.iter().map(|a| a.id).collect::<Vec<_>>();

    if binds_to_delete.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Binds Deletion Failed")
            .description("There were no binds found associated with given ids")
            .build()
            .unwrap();
        ctx.respond().embeds(&[embed]).exec().await?;
        return Ok(());
    }

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"AssetBinds": {"_id": {"$in": bind_ids}}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Deletion Successful!")
        .description("The given binds were successfully deleted")
        .build()
        .unwrap();
    let message = ctx
        .respond()
        .embeds(&[embed])
        .components(&[Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "↩️".into()
                }),
                label: Some("Uh oh? Revert".into()),
                custom_id: Some("ab-delete-revert".into()),
                url: None,
                disabled: false,
            })],
        })])
        .exec()
        .await?
        .model()
        .await?;

    let ids_str = binds_to_delete
        .iter()
        .map(|b| format!("`Id`: {}\n", b.id))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Asset Bind Deletion")
        .field(EmbedFieldBuilder::new("Assets Deleted", ids_str))
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
                    let filter = doc! {"_id": guild.id};
                    let update =
                        doc! {"$push": {"AssetBinds": {"$each": bson::to_bson(&binds_to_delete)?}}};
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
