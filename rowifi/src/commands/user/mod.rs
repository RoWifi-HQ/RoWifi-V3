mod info;
mod test;
mod update;
mod verify;

use rowifi_framework::prelude::*;
use rowifi_models::discord::id::{marker::MessageMarker, Id};

pub use info::{botinfo, support, userinfo};
pub use test::test;
pub use update::update;
pub use verify::{verify, verify_config};

use crate::commands::user::update::{update_func, UpdateArguments};

pub fn user_config(cmds: &mut Vec<Command>) {
    let update_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["update", "getroles"])
        .description("Command to update an user")
        .group("User")
        .handler(update);

    let userinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["userinfo"])
        .description("Command to view information about an user")
        .group("User")
        .handler(userinfo);

    let botinfo_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["botinfo"])
        .description("Command to view information about the bot")
        .group("User")
        .handler(botinfo);

    let support_cmd = Command::builder()
        .level(RoLevel::Normal)
        .names(&["support", "invite"])
        .description("View important links related to the bot")
        .group("User")
        .handler(support);

    let test_cmd = Command::builder()
        .level(RoLevel::Creator)
        .names(&["test"])
        .handler(test);

    cmds.push(update_cmd);
    cmds.push(userinfo_cmd);
    cmds.push(botinfo_cmd);
    cmds.push(support_cmd);
    cmds.push(test_cmd);

    verify_config(cmds);
}

pub async fn handle_update_button(
    ctx: &CommandContext,
    message_id: Id<MessageMarker>,
    keep_components: Vec<Component>,
) -> Result<(), RoError> {
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
                if component_interaction_author == author_id
                    && message_component.data.custom_id == "handle-update"
                {
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
                                        .components(keep_components)
                                        .build(),
                                ),
                            },
                        )
                        .exec()
                        .await?;

                    let embed = update_func(ctx, UpdateArguments { user_id: None }, false).await?;
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
