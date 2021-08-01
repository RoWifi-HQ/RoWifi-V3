#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::let_underscore_drop,
    clippy::too_many_lines,
    clippy::must_use_candidate,
    clippy::non_ascii_literal,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::similar_names
)]

pub mod arguments;
pub mod bucket;
pub mod command;
pub mod context;
pub mod error;
pub mod extensions;
pub mod handler;
mod parser;
pub mod prelude;
pub mod respond;
pub mod utils;

use futures_util::future::{ready, Either, Ready};
use itertools::Itertools;
use rowifi_cache::{CachedGuild, CachedMember};
use std::{
    future::Future,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc},
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{application_command::CommandDataOption, Interaction},
    },
    channel::{message::MessageFlags, Message},
    gateway::event::Event,
    guild::Permissions,
    id::{GuildId, UserId},
};
use uwl::Stream;

use arguments::Arguments;
use command::{Command, CommandResult};
use context::{BotContext, CommandContext};
use error::RoError;
use extensions::EmbedExtensions;
use parser::PrefixType;
use utils::RoLevel;

pub use framework_derive::FromArgs;

#[allow(clippy::large_enum_variant)]
pub enum ServiceRequest {
    Message(Arguments),
    Interaction(Vec<CommandDataOption>),
    Help(Arguments, EmbedBuilder),
}

pub struct Framework {
    bot: BotContext,
    cmds: Vec<Command>,
    default_perms: Permissions,
}

impl Framework {
    pub fn new(bot: BotContext, default_perms: Permissions) -> Self {
        Self {
            bot,
            cmds: Vec::new(),
            default_perms,
        }
    }

    pub fn command(mut self, cmd: Command) -> Self {
        self.cmds.push(cmd);
        self
    }

    pub fn configure<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut Vec<Command>),
    {
        func(&mut self.cmds);
        self
    }

    fn help(
        &mut self,
        msg: &Message,
        mut args: Arguments,
    ) -> Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>> {
        let mut embed = EmbedBuilder::new().default_data().title("Help");

        if let Some(arg) = args.next() {
            if let Some(cmd) = self.cmds.iter_mut().find(|c| c.names.contains(&arg)) {
                let ctx = CommandContext {
                    bot: self.bot.clone(),
                    channel_id: msg.channel_id,
                    guild_id: msg.guild_id,
                    author: Arc::new(msg.author.clone()),
                    message_id: Some(msg.id),
                    interaction_id: None,
                    interaction_token: None,
                    callback_invoked: Arc::new(AtomicBool::new(false)),
                };
                let req = ServiceRequest::Help(args, embed);
                return cmd.call((ctx, req));
            }
        }

        embed = embed.description("Listing all top-level commands");
        let groups = self
            .cmds
            .iter()
            .sorted_by_key(|c| c.options.group)
            .group_by(|c| c.options.group);
        for (group, commands) in &groups {
            if let Some(group) = group {
                let commands = commands
                    .filter(|c| !c.options.hidden)
                    .map(|m| format!("`{}`", m.names[0]))
                    .join(" ");
                embed = embed.field(EmbedFieldBuilder::new(group, commands));
            }
        }
        let embed = embed.build().unwrap();
        let bot = self.bot.clone();
        let channel_id = msg.channel_id;
        let fut = async move {
            bot.http
                .create_message(channel_id)
                .embeds(vec![embed])
                .unwrap()
                .await?;
            Ok(())
        };
        Box::pin(fut)
    }
}

#[allow(clippy::type_complexity)]
impl Service<&Event> for Framework {
    type Response = ();
    type Error = RoError;
    type Future = Either<
        Ready<Result<(), Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: &Event) -> Self::Future {
        match req {
            Event::MessageCreate(msg) => {
                let mut stream = Stream::new(&msg.content);
                stream.take_while_char(char::is_whitespace);

                let prefix = parser::find_prefix(&mut stream, &self.bot, msg.guild_id);
                if let Some(PrefixType::Mention) = prefix {
                    if let Some(guild_id) = msg.guild_id {
                        if stream.rest().is_empty()
                            && !self.bot.disabled_channels.contains(&msg.channel_id)
                        {
                            let actual_prefix = self
                                .bot
                                .prefixes
                                .get(&guild_id)
                                .map_or_else(|| self.bot.default_prefix.clone(), |p| p.to_string());
                            let http = self.bot.http.clone();
                            let channel_id = msg.channel_id;
                            tokio::spawn(async move {
                                let _ = http
                                    .create_message(channel_id)
                                    .content(format!("My prefix here is {}", actual_prefix))
                                    .unwrap()
                                    .await;
                            });
                            return Either::Left(ready(Ok(())));
                        }
                    }
                }

                if prefix.is_none() {
                    return Either::Left(ready(Ok(())));
                }

                let content = stream.rest().to_string();
                let mut cmd_str = Arguments::new(&content);

                let command = if let Some(arg) = cmd_str.next() {
                    if arg.eq_ignore_ascii_case("help")
                        && !self.bot.disabled_channels.contains(&msg.channel_id)
                    {
                        return Either::Right(self.help(msg, cmd_str));
                    }
                    self.cmds
                        .iter_mut()
                        .find(|c| c.names.iter().any(|c| c.eq_ignore_ascii_case(arg)))
                } else {
                    None
                };

                let command = match command {
                    Some(c) => c,
                    None => return Either::Left(ready(Ok(()))),
                };

                match self.bot.cache.channel_permissions(msg.channel_id) {
                    Some(p) => {
                        if !p.contains(self.default_perms)
                            && !p.contains(Permissions::ADMINISTRATOR)
                        {
                            let http = self.bot.http.clone();
                            let perms = self.default_perms;
                            let channel_id = msg.channel_id;
                            let fut = async move {
                                let _ = http.create_message(channel_id)
                                    .content(format!(
                                        "I seem to be missing one of the following permissions: `{:?}`",
                                        perms
                                    ))
                                    .unwrap()
                                    .await;
                                Ok(())
                            };
                            return Either::Right(Box::pin(fut));
                        }
                    }
                    None => return Either::Left(ready(Ok(()))),
                }

                if !run_checks(&self.bot, command, msg.guild_id, msg.author.id) {
                    return Either::Left(ready(Ok(())));
                }

                let ctx = CommandContext {
                    bot: self.bot.clone(),
                    channel_id: msg.channel_id,
                    guild_id: msg.guild_id,
                    author: Arc::new(msg.author.clone()),
                    message_id: Some(msg.id),
                    interaction_id: None,
                    interaction_token: None,
                    callback_invoked: Arc::new(AtomicBool::new(false)),
                };

                let request = ServiceRequest::Message(cmd_str);
                let cmd_fut = command.call((ctx, request));
                return Either::Right(cmd_fut);
            }
            Event::InteractionCreate(interaction) => {
                if let Interaction::ApplicationCommand(top_command) = &interaction.0 {
                    let user = match top_command.member.clone().and_then(|m| m.user) {
                        Some(u) => u,
                        None => return Either::Left(ready(Ok(()))),
                    };
                    let command_options = &top_command.data.options;
                    let command = self
                        .cmds
                        .iter_mut()
                        .find(|c| c.names.contains(&top_command.data.name.as_str()));
                    let command = match command {
                        Some(c) => c,
                        None => return Either::Left(ready(Ok(()))),
                    };
                    let id = top_command.id;
                    let token = top_command.token.clone();

                    if !run_checks(&self.bot, command, top_command.guild_id, user.id) {
                        let http = self.bot.http.clone();
                        let fut = async move {
                            let _ = http
                                .interaction_callback(
                                    id,
                                    token,
                                    InteractionResponse::ChannelMessageWithSource(CallbackData {
                                        allowed_mentions: None,
                                        tts: None,
                                        embeds: Vec::new(),
                                        content: Some(
                                            "You do not have sufficient perms to run this command"
                                                .into(),
                                        ),
                                        flags: Some(MessageFlags::EPHEMERAL),
                                        components: None,
                                    }),
                                )
                                .await;
                            Ok(())
                        };
                        return Either::Right(Box::pin(fut));
                    }

                    let ctx = CommandContext {
                        bot: self.bot.clone(),
                        channel_id: top_command.channel_id,
                        guild_id: top_command.guild_id,
                        author: Arc::new(user),
                        message_id: None,
                        interaction_id: Some(id),
                        interaction_token: Some(top_command.token.clone()),
                        callback_invoked: Arc::new(AtomicBool::new(false)),
                    };

                    let request = ServiceRequest::Interaction(command_options.clone());
                    let cmd_fut = command.call((ctx, request));
                    return Either::Right(cmd_fut);
                } else if let Interaction::MessageComponent(message_component) = &interaction.0 {
                    if !self
                        .bot
                        .ignore_message_components
                        .contains(&message_component.message.id)
                    {
                        let http = self.bot.http.clone();
                        let id = message_component.id;
                        let token = message_component.token.clone();
                        let fut = async move {
                            let _ = http
                                .interaction_callback(
                                    id,
                                    &token,
                                    InteractionResponse::DeferredUpdateMessage,
                                )
                                .await;
                            let _ = http
                                .create_followup_message(&token)
                                .unwrap()
                                .ephemeral(true)
                                .content("This component is no longer active and cannot be used.")
                                .await;
                            Ok(())
                        };
                        return Either::Right(Box::pin(fut));
                    }
                }
            }
            _ => {}
        }
        let fut = ready(Ok(()));
        Either::Left(fut)
    }
}

fn run_checks(bot: &BotContext, cmd: &Command, guild_id: Option<GuildId>, author: UserId) -> bool {
    if bot.owners.contains(&author) {
        return true;
    }

    if let Some(guild_id) = guild_id {
        if let Some(guild) = bot.cache.guild(guild_id) {
            if let Some(member) = bot.cache.member(guild_id, author) {
                let level = get_perm_level(bot, &guild, &member);
                return cmd.options.level <= level;
            }
        }
    }

    false
}

fn get_perm_level(bot: &BotContext, guild: &CachedGuild, member: &CachedMember) -> RoLevel {
    if bot.owners.contains(&member.user.id) {
        return RoLevel::Creator;
    }

    if member.user.id == guild.owner_id {
        return RoLevel::Admin;
    }

    if let Some(admin_role) = guild.admin_role {
        if member.roles.contains(&admin_role) {
            return RoLevel::Admin;
        }
    }

    if let Some(admin_roles) = bot.admin_roles.get(&guild.id) {
        for admin_role in admin_roles.value() {
            if member.roles.contains(admin_role) {
                return RoLevel::Admin;
            }
        }
    }

    for role in &member.roles {
        if let Some(role) = bot.cache.role(*role) {
            if role.permissions.contains(Permissions::ADMINISTRATOR) {
                return RoLevel::Admin;
            }
        }
    }

    if let Some(trainer_role) = guild.trainer_role {
        if member.roles.contains(&trainer_role) {
            return RoLevel::Trainer;
        }
    }

    if let Some(trainer_roles) = bot.trainer_roles.get(&guild.id) {
        for trainer_role in trainer_roles.value() {
            if member.roles.contains(trainer_role) {
                return RoLevel::Trainer;
            }
        }
    }

    RoLevel::Normal
}
