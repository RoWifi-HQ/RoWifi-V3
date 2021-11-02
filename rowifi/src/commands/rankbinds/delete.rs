use itertools::Itertools;
use mongodb::bson::{self, doc};
use rowifi_framework::prelude::*;
use std::str::FromStr;

#[derive(FromArgs)]
pub struct RankBindsDelete {
    #[arg(help = "The Group ID of the Rankbind to delete")]
    pub group_id: i64,
    #[arg(help = "The Rank ID of the Rankbind to delete", rest)]
    pub rank_id: String,
}

pub async fn rankbinds_delete(ctx: CommandContext, args: RankBindsDelete) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0.get()).await?;

    let group_id = args.group_id;

    let mut rank_ids_to_delete = Vec::new();
    for arg in args.rank_id.split_ascii_whitespace() {
        if let Ok(r) = RankId::from_str(arg) {
            rank_ids_to_delete.push(r);
        }
    }

    let mut binds_to_delete = Vec::new();
    for rank in rank_ids_to_delete {
        match rank {
            RankId::Range(r1, r2) => {
                let binds = guild
                    .rankbinds
                    .iter()
                    .filter(|r| r.group_id == group_id && r.rank_id >= r1 && r.rank_id <= r2);
                binds_to_delete.extend(binds);
            }
            RankId::Single(rank) => {
                if let Some(b) = guild
                    .rankbinds
                    .iter()
                    .find(|r| r.group_id == group_id && r.rank_id == rank)
                {
                    binds_to_delete.push(b);
                }
            }
        }
    }
    let bind_ids = binds_to_delete
        .iter()
        .map(|r| r.rbx_rank_id)
        .unique()
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

    let filter = doc! {"_id": guild.id};
    let update = doc! {"$pull": {"RankBinds": {"RbxGrpRoleId": {"$in": bind_ids}}}};
    ctx.bot.database.modify_guild(filter, update).await?;

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
                custom_id: Some("rb-delete-revert".into()),
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
        .map(|r| format!("`Id`: {}\n", r.rbx_rank_id))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Rank Bind Deletion")
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
                    let filter = doc! {"_id": guild.id};
                    let update =
                        doc! {"$push": {"RankBinds": {"$each": bson::to_bson(&binds_to_delete)?}}};
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
