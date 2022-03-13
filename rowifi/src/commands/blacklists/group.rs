use rowifi_framework::prelude::*;
use rowifi_models::blacklist::{Blacklist, BlacklistData};

#[derive(FromArgs)]
pub struct BlacklistGroupArguments {
    #[arg(help = "The Group ID to blacklist")]
    pub group_id: i64,
    #[arg(help = "The reason of the blacklist", rest)]
    pub reason: String,
}

pub async fn blacklist_group(ctx: CommandContext, args: BlacklistGroupArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let group_id = args.group_id;
    let mut reason = args.reason;
    if reason.is_empty() {
        reason = "N/A".into();
    }

    let blacklist_id = guild
        .blacklists
        .iter()
        .map(|b| b.blacklist_id)
        .max()
        .unwrap_or_default()
        + 1;
    let blacklist = Blacklist {
        blacklist_id,
        reason,
        data: BlacklistData::Group(group_id),
    };
    ctx.bot
        .database
        .execute(
            r#"UPDATE guilds SET blacklists = array_append(blacklists, $1) WHERE guild_id = $2"#,
            &[&blacklist, &(guild_id)],
        )
        .await?;

    let name = format!("Type: {:?}", blacklist.kind());
    let desc = format!("Group Id: {}\nReason: {}", group_id, blacklist.reason);

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Blacklist Addition Successful")
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .color(Color::DarkGreen as u32)
        .build();
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
                custom_id: Some("bl-group-delete".into()),
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
        .description("Blacklist Addition")
        .field(EmbedFieldBuilder::new(name, desc))
        .build();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = message.id;
    let author_id = ctx.author.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
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
                        .create_response(
                            message_component.id,
                            &message_component.token,
                            &InteractionResponse {
                                kind: InteractionResponseType::UpdateMessage,
                                data: Some(
                                    InteractionResponseDataBuilder::new()
                                        .components(Vec::new())
                                        .build(),
                                ),
                            },
                        )
                        .exec()
                        .await?;

                    ctx.bot.database.execute("UPDATE guilds SET blacklists = array_remove(blacklists, $1) WHERE guild_id = $2", &[&blacklist, &(guild_id)]).await?;

                    let embed = EmbedBuilder::new()
                        .default_data()
                        .color(Color::DarkGreen as u32)
                        .title("Successful!")
                        .description("The newly created blacklist was deleted")
                        .build();
                    ctx.bot
                        .http
                        .interaction(ctx.bot.application_id)
                        .create_followup(&message_component.token)
                        .embeds(&[embed])?
                        .exec()
                        .await?;

                    break;
                }
                let _ = ctx
                    .bot
                    .http
                    .interaction(ctx.bot.application_id)
                    .create_response(
                        message_component.id,
                        &message_component.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::DeferredUpdateMessage,
                            data: None,
                        },
                    )
                    .exec()
                    .await;
                let _ = ctx
                    .bot
                    .http
                    .interaction(ctx.bot.application_id)
                    .create_followup(&message_component.token)
                    .flags(MessageFlags::EPHEMERAL)
                    .content("This button is only interactable by the original command invoker")?
                    .exec()
                    .await;
            }
        }
    }
    ctx.bot.ignore_message_components.remove(&message_id);

    Ok(())
}
