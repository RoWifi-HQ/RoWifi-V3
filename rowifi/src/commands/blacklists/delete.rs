use mongodb::bson::{self, doc};
use rowifi_framework::prelude::*;

#[derive(FromArgs)]
pub struct BlacklistDeleteArguments {
    #[arg(help = "The ID of the blacklist to delete", rest)]
    pub id: String,
}

pub async fn blacklist_delete(
    ctx: CommandContext,
    args: BlacklistDeleteArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let id = args.id;
    let blacklist = match guild.blacklists.iter().find(|b| b.id == id) {
        Some(b) => b,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Blacklist Deletion Failed")
                .description("A blacklist with the given id was not found")
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"Blacklists": {"_id": id}}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let embed = EmbedBuilder::new()
        .default_data()
        .color(Color::DarkGreen as u32)
        .title("Blacklist Deletion Successful")
        .description("The given blacklist was successfully deleted")
        .build()
        .unwrap();
    let message_id = ctx
        .respond()
        .embed(embed)
        .component(Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "↩️".into()
                }),
                label: Some("Uh oh? Revert".into()),
                custom_id: Some("bl-delete-revert".into()),
                url: None,
                disabled: false,
            })],
        }))
        .await?;

    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Blacklist Deletion")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = message_id.unwrap();
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
                    let update = doc! {"$push": {"Blacklists": bson::to_bson(blacklist)?}};
                    ctx.bot.database.modify_guild(filter, update).await?;
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(Vec::new()),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Restoration Successful!")
                        .description("The deleted blacklist was successfully restored")
                        .build()
                        .unwrap();
                    ctx.bot
                        .http
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .embeds(vec![embed])
                        .await?;

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
                    .content("This button is only interactable by the original command invoker")
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
