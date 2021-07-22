use itertools::Itertools;
use mongodb::bson::{doc, to_bson};
use rowifi_framework::prelude::*;
use rowifi_models::{
    bind::{CustomBind, Template},
    guild::RoGuild,
    roblox::id::UserId as RobloxUserId,
    rolang::{RoCommand, RoCommandUser},
};
use std::collections::HashMap;
use twilight_model::id::{GuildId, RoleId};

#[derive(FromArgs)]
pub struct CustombindsNewArguments {
    #[arg(help = "The code that makes up the bind", rest)]
    pub code: String,
}

pub async fn custombinds_new(ctx: CommandContext, args: CustombindsNewArguments) -> CommandResult {
    let guild_id = ctx.guild_id.unwrap();
    let guild = ctx.bot.database.get_guild(guild_id.0).await?;

    let code = args.code;

    custombinds_new_common(ctx, guild_id, guild, code).await
}

pub async fn custombinds_new_common(
    ctx: CommandContext,
    guild_id: GuildId,
    guild: RoGuild,
    code: String,
) -> CommandResult {
    let user = match ctx.get_linked_user(ctx.author.id, guild_id).await? {
        Some(u) => u,
        None => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Addition Failed")
                .description("You must be verified to create a custom bind")
                .build()
                .unwrap();
            ctx.respond().embed(embed).await?;
            return Ok(());
        }
    };
    let user_id = RobloxUserId(user.roblox_id as u64);
    let member = ctx
        .member(ctx.guild_id.unwrap(), ctx.author.id.0)
        .await?
        .unwrap();
    let ranks = ctx
        .bot
        .roblox
        .get_user_roles(user_id)
        .await?
        .iter()
        .map(|r| (r.group.id.0 as i64, i64::from(r.role.rank)))
        .collect::<HashMap<_, _>>();
    let roblox_user = ctx.bot.roblox.get_user(user_id).await?;

    let command_user = RoCommandUser {
        user: &user,
        roles: &member.roles,
        ranks: &ranks,
        username: &roblox_user.name,
    };
    let command = match RoCommand::new(&code) {
        Ok(c) => c,
        Err(s) => {
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .content(s)
                .unwrap()
                .await?;
            return Ok(());
        }
    };
    if let Err(res) = command.evaluate(&command_user) {
        ctx.bot
            .http
            .create_message(ctx.channel_id)
            .content(res)
            .unwrap()
            .await?;
        return Ok(());
    }

    let template = await_reply(
        "Enter the template you wish to set for the bind.\nYou may also enter `N/A`, `disable`",
        &ctx,
    )
    .await?;
    let template_str = match template.as_str() {
        "disable" => "{discord-name}".into(),
        "N/A" => "{roblox-username}".into(),
        _ => {
            if Template::has_slug(template.as_str()) {
                template.clone()
            } else {
                format!("{} {{roblox-username}}", template)
            }
        }
    };

    let priority = match await_reply("Enter the priority you wish to set for the bind.", &ctx)
        .await?
        .parse::<i64>()
    {
        Ok(p) => p,
        Err(_) => {
            let embed = EmbedBuilder::new()
                .default_data()
                .color(Color::Red as u32)
                .title("Custom Bind Addition Failed")
                .description("Expected priority to be a number")
                .build()
                .unwrap();
            ctx.bot
                .http
                .create_message(ctx.channel_id)
                .embeds(vec![embed])
                .unwrap()
                .await?;
            return Ok(());
        }
    };

    let server_roles = ctx.bot.cache.roles(guild_id);
    let discord_roles_str = await_reply("Enter the roles you wish to set for the bind.\nEnter `N/A` if you would not like to set roles. Please tag the roles to ensure the bot can recognize them.", &ctx).await?;
    let mut discord_roles = Vec::new();
    for role_str in discord_roles_str.split_ascii_whitespace() {
        if let Some(role_id) = parse_role(role_str) {
            if server_roles.contains(&RoleId(role_id)) {
                discord_roles.push(role_id as i64);
            }
        }
    }

    let mut binds = guild.custombinds.iter().map(|c| c.id).collect_vec();
    binds.sort_unstable();
    let id = binds.last().unwrap_or(&0) + 1;
    let bind = CustomBind {
        id,
        code: code.clone(),
        prefix: None,
        priority,
        command,
        discord_roles,
        template: Some(Template(template_str)),
    };
    let bind_bson = to_bson(&bind)?;
    let filter = doc! {"_id": guild.id};
    let update = doc! {"$push": {"CustomBinds": &bind_bson}};
    ctx.bot.database.modify_guild(filter, update).await?;

    let name = format!("Id: {}", bind.id);
    let roles_str = bind
        .discord_roles
        .iter()
        .map(|r| format!("<@&{}> ", r))
        .collect::<String>();
    let desc = format!(
        "Code: {}\nTemplate: `{}`\nPriority: {}\nDiscord Roles: {}",
        bind.code,
        bind.template.unwrap(),
        bind.priority,
        roles_str
    );
    let embed = EmbedBuilder::new()
        .default_data()
        .title("Bind Addition Successful")
        .color(Color::DarkGreen as u32)
        .field(EmbedFieldBuilder::new(name.clone(), desc.clone()))
        .build()
        .unwrap();
    let message = ctx
        .bot
        .http
        .create_message(ctx.channel_id)
        .embeds(vec![embed])
        .unwrap()
        .components(vec![Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                style: ButtonStyle::Danger,
                emoji: Some(ReactionType::Unicode {
                    name: "🗑️".into()
                }),
                label: Some("Oh no! Delete?".into()),
                custom_id: Some("cb-new-delete".into()),
                url: None,
                disabled: false,
            })],
        })])
        .unwrap()
        .await?;

    let log_embed = EmbedBuilder::new()
        .default_data()
        .title(format!("Action by {}", ctx.author.name))
        .description("Custom Bind Addition")
        .field(EmbedFieldBuilder::new(name, desc))
        .build()
        .unwrap();
    ctx.log_guild(guild_id, log_embed).await;

    let author_id = ctx.author.id;
    let message_id = message.id;

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
                    let update = doc! {"$pull": {"CustomBinds": bind_bson}};
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
                        .description("The newly created bind was deleted")
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

    ctx.bot
        .http
        .update_message(ctx.channel_id, message_id)
        .components(None)
        .unwrap()
        .await?;
    Ok(())
}
