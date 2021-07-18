use rowifi_framework::prelude::*;

#[allow(dead_code, unused_variables)]
pub async fn bind(ctx: CommandContext) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let message_id = ctx
        .respond()
        .component(Component::ActionRow(ActionRow {
            components: vec![Component::SelectMenu(SelectMenu {
                custom_id: "bind-select".into(),
                max_values: Some(1),
                min_values: Some(1),
                options: vec![
                    SelectMenuOption {
                        default: false,
                        description: Some("Used to bind a Roblox asset, badge or gamepass".into()),
                        emoji: None,
                        label: "Asset".into(),
                        value: "asset".into(),
                    },
                    SelectMenuOption {
                        default: false,
                        description: Some(
                            "Use our powerful Lua-like language to create your own bind".into(),
                        ),
                        emoji: None,
                        label: "Custom".into(),
                        value: "custom".into(),
                    },
                    SelectMenuOption {
                        default: false,
                        description: Some("Bind the ranks of a Roblox Group".into()),
                        emoji: None,
                        label: "Group".into(),
                        value: "group".into(),
                    },
                ],
                placeholder: None,
            })],
        }))
        .content("What type of bind would you like to create?")
        .await?;

    let message_id = message_id.unwrap();
    let author_id = ctx.author.id;
    let stream = ctx
        .bot
        .standby
        .wait_for_component_interaction(message_id)
        .timeout(Duration::from_secs(300));
    tokio::pin!(stream);

    // let mut bind_type = None;
    while let Some(Ok(event)) = stream.next().await {
        if let Event::InteractionCreate(interaction) = &event {
            if let Interaction::MessageComponent(message_component) = &interaction.0 {
                let component_interaction_author = message_component
                    .as_ref()
                    .member
                    .as_ref()
                    .unwrap()
                    .user
                    .as_ref()
                    .unwrap()
                    .id;
                if component_interaction_author == author_id {
                    // bind_type =
                }
            }
        }
    }

    todo!("Complete once the disabled PR on twilight is merged");
}
