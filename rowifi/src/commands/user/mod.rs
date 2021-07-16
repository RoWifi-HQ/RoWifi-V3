mod info;
mod test;
mod update;
mod verify;

pub use info::{botinfo, support, userinfo};
use rowifi_framework::prelude::*;
pub use test::test;
use twilight_model::id::MessageId;
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
    message_id: MessageId,
    keep_components: Vec<Component>,
) -> Result<(), RoError> {
    let author_id = ctx.author.id;

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
                    .member
                    .as_ref()
                    .and_then(|m| m.user.as_ref())
                    .map(|u| u.id)
                    .unwrap();
                if component_interaction_author == author_id
                    && message_component.data.custom_id == "handle-update"
                {
                    ctx.bot
                        .http
                        .interaction_callback(
                            message_component.id,
                            &message_component.token,
                            InteractionResponse::UpdateMessage(CallbackData {
                                allowed_mentions: None,
                                content: None,
                                components: Some(keep_components),
                                embeds: Vec::new(),
                                flags: None,
                                tts: None,
                            }),
                        )
                        .await?;

                    let embed = update_func(ctx, UpdateArguments { user_id: None }).await?;
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
