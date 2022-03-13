use rowifi_framework::prelude::*;
use rowifi_models::{
    blacklist::{Blacklist, BlacklistData},
    id::UserId,
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;

#[derive(FromArgs)]
pub struct BlacklistCustomArguments {
    #[arg(help = "Code to use in the blacklist", rest)]
    pub code: String,
}

pub async fn blacklist_custom(
    ctx: CommandContext,
    args: BlacklistCustomArguments,
) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id).await?;

    let code = args.code;
    if code.is_empty() {
        let embed = EmbedBuilder::new()
            .default_data()
            .color(Color::Red as u32)
            .title("Custom Blacklist Addition Failed")
            .description("No code was found. Please try again")
            .build();
        ctx.respond().embeds(&[embed])?.exec().await?;
        return Ok(());
    }
    let user = match ctx
        .bot
        .database
        .get_linked_user(UserId(ctx.author.id), guild_id)
        .await?
    {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Blacklist Addition Failed")
                .description("You must be verified to create a custom blacklist")
                .build();
            ctx.respond().embeds(&[embed])?.exec().await?;
            return Ok(());
        }
    };
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx.member(guild_id, UserId(ctx.author.id)).await?.unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id, false).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            ctx.respond().content(&s)?.exec().await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.respond().content(&res)?.exec().await?;
        return Ok(());
    }
    let reason = await_reply("Enter the reason of this blacklist.", &ctx).await?;

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
        data: BlacklistData::Custom(command),
    };
    ctx.bot
        .database
        .execute(
            r#"UPDATE guilds SET blacklists = array_append(blacklists, $1) WHERE guild_id = $2"#,
            &[&blacklist, &(guild_id)],
        )
        .await?;

    let name = format!("Type: {:?}", blacklist.kind());
    let desc = format!("Code: {}\nReason: {}", code, blacklist.reason);

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
                custom_id: Some("bl-custom-delete".into()),
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
