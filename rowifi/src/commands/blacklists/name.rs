use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::blacklist::{Blacklist, BlacklistType};

#[derive(FromArgs)]
pub struct BlacklistNameArguments {
    #[arg(help = "The username to blacklist. This will get converted into the id in the database")]
    pub username: String,
    #[arg(help = "The reason of the blacklist", rest)]
    pub reason: String,
}

pub async fn blacklist_name(ctx: CommandContext, args: BlacklistNameArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let username = args.username;
    let user = match ctx.bot.roblox.get_user_from_username(&username).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Blacklist Addition Failed")
                .description(format!(
                    "There was no user found with username {}",
                    username
                ))
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };

    let mut reason = args.reason;
    if reason.is_empty() {
        reason = "N/A".into();
    }

    let blacklist = Blacklist {
        id: user.id.0.to_string(),
        reason,
        blacklist_type: BlacklistType::Name(user.id.0.to_string()),
    };
    let blacklist_bson = to_bson(&blacklist)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"Blacklists": &blacklist_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Type: {:?}", blacklist.blacklist_type);
    let desc = format!("Id: {}\nReason: {}", blacklist.id, blacklist.reason);

    let embed = EmbedBuilder::new()
        .default_data()
        .title("Blacklist Addition Successful")
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .color(Color::DarkGreen as u32)
        .build()
        .unwrap();
    let message_id = ctx
        .respond()
        .embed(embed)
        .component(Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "🗑️".into()
                }),
                label: Some("Oh no! Delete?".into()),
                custom_id: Some("bl-name-delete".into()),
                url: None,
                disabled: false,
            })],
        }))
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Blacklist Addition")
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

    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component.author_id().unwrap();
                if component_interaction_author == author_id {
                    let filter = doc! {"_id": guild.id};
                    let update = doc! {"$pull": {"Blacklists": blacklist_bson}};
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
                        .title("Successful!")
                        .description("The newly created blacklist was deleted")
                        .build()
                        .unwrap();
                    ctx.bot
                        .http
                        .create_followup_message(&message_component.token)
                        .unwrap()
                        .embeds(vec![embed])
                        .await?;

                    return Ok(());
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
