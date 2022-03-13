use itertools::Itertools;
use rowifi_database::dynamic_args;
use rowifi_framework::prelude::*;
use rowifi_models::bind::{BindType, Rankbind};
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
    let rankbinds = ctx
        .bot
        .database
        .query::<Rankbind>(
            "SELECT * FROM binds WHERE guild_id = $1 AND bind_type = $2",
            &[&(guild_id), &BindType::Rank],
        )
        .await?;

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
                let binds = rankbinds.iter().filter(|r| {
                    r.group_id == group_id && r.group_rank_id >= r1 && r.group_rank_id <= r2
                });
                binds_to_delete.extend(binds);
            }
            RankId::Single(rank) => {
                if let Some(b) = rankbinds
                    .iter()
                    .find(|r| r.group_id == group_id && r.group_rank_id == rank)
                {
                    binds_to_delete.push(b);
                }
            }
        }
    }
    let bind_ids = binds_to_delete
        .iter()
        .map(|r| &r.bind_id)
        .unique()
        .collect::<Vec<_>>();

    if binds_to_delete.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Binds Deletion Failed")
            .description("There were no binds found associated with given ids")
            .build();
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
        .build();
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
        .map(|r| format!("`Id`: {}\n", r.roblox_rank_id))
        .collect::<String>();
    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Rank Bind Deletion")
        .field(EmbedFieldBuilder::new("Binds Deleted", ids_str))
        .build();
    ctx.log_guild(guild_id, log_embed).await;

    let message_id = message.id;
    let author_id = ctx.author.id;

    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(30));
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
                                        .build()
                                )
                            }
                        )
                        .exec()
                        .await?;

                    let mut db = ctx.bot.database.get().await?;
                    let transaction = db.transaction().await?;
                    let stmt = transaction.prepare_cached("INSERT INTO binds(bind_type, guild_id, group_id, group_rank_id, roblox_rank_id, template, priority, discord_roles) VALUES($1, $2, $3, $4, $5, $6, $7, $8)").await?;
                    for bind in binds_to_delete {
                        transaction
                            .execute(
                                &stmt,
                                &[
                                    &BindType::Rank,
                                    &(guild_id),
                                    &bind.group_id,
                                    &bind.group_rank_id,
                                    &bind.roblox_rank_id,
                                    &bind.template,
                                    &bind.priority,
                                    &bind.discord_roles,
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
                        .build();
                    ctx.bot
                        .http
                        .interaction(ctx.bot.application_id)
                        .create_followup(&message_component.token)
                        .embeds(&[embed])?
                        .exec()
                        .await?;

                    ctx.bot.ignore_message_components.remove(&message_id);
                    return Ok(());
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
                            data: None
                        }
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
