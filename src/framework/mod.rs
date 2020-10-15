pub mod context;
mod map;
pub mod parser;
pub mod prelude;
pub mod structures;

use crate::utils::error::{CommandError, RoError};
use dashmap::DashMap;
use lazy_static::lazy_static;
use std::time::Duration;
use transient_dashmap::TransientDashMap;
use twilight_command_parser::Arguments;
use twilight_gateway::Event;
use twilight_model::{channel::Message, guild::Permissions};
use uwl::Stream;

use context::Context;
pub use map::CommandMap;
use parser::{Invoke, ParseError};
use structures::*;

lazy_static! {
    static ref ROWIFI_PERMS: Permissions = Permissions::SEND_MESSAGES
        | Permissions::EMBED_LINKS
        | Permissions::MANAGE_ROLES
        | Permissions::MANAGE_NICKNAMES
        | Permissions::ADD_REACTIONS;
}

#[derive(Default)]
pub struct Framework {
    commands: Vec<(&'static Command, CommandMap)>,
    buckets: DashMap<String, Bucket>,
    help: Option<&'static HelpCommand>,
}

impl Framework {
    pub fn command(mut self, command: &'static Command) -> Self {
        let map = CommandMap::new(&[command]);
        self.commands.push((command, map));
        self
    }

    pub fn help(mut self, help: &'static HelpCommand) -> Self {
        self.help = Some(help);
        self
    }

    pub fn bucket(self, name: &str, time: Duration, calls: u64) -> Self {
        self.buckets.insert(
            name.to_string(),
            Bucket {
                time,
                guilds: TransientDashMap::new(time),
                calls,
            },
        );
        self
    }

    async fn dispatch(&self, msg: Message, context: &Context) {
        if msg.author.bot
            || msg.webhook_id.is_some()
            || msg.guild_id.is_none()
            || msg.content.is_empty()
        {
            return;
        }

        let mut stream = Stream::new(&msg.content);
        stream.take_while_char(|c| c.is_whitespace());

        let prefix = parser::find_prefix(&mut stream, &msg, context.config.as_ref());
        if prefix.is_some() && stream.rest().is_empty() {
            let actual_prefix = if let Some(p) = context.config.prefixes.get(&msg.guild_id.unwrap())
            {
                p.value().to_owned()
            } else {
                context.config.default_prefix.clone()
            };
            let _ = context
                .http
                .create_message(msg.channel_id)
                .content(format!("My prefix here is {}", actual_prefix))
                .unwrap()
                .await;
            return;
        }

        if prefix.is_none() {
            return;
        }

        match context.cache.channel_permissions(msg.channel_id) {
            Some(p) => {
                if !p.contains(*ROWIFI_PERMS) && !p.contains(Permissions::ADMINISTRATOR) {
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content(format!(
                            "I seem to be missing one of the following permissions: `{:?}`",
                            *ROWIFI_PERMS
                        ))
                        .unwrap()
                        .await;
                    return;
                }
            }
            None => return,
        }

        let invocation = parser::command(
            &mut stream,
            &self.commands,
            &self.help.as_ref().map(|h| h.name),
        );
        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(_)) => {
                return;
            }
        };

        match invoke {
            Invoke::Help => {
                if context.config.disabled_channels.contains(&msg.channel_id) {
                    return;
                }
                let args = Arguments::new(stream.rest());
                if let Some(help) = self.help {
                    let _res = (help.fun)(context, &msg, args, &self.commands).await;
                }
            }
            Invoke::Command { command } => {
                if !self.run_checks(&context, &msg, command) {
                    return;
                }

                if let Some(bucket) = command.options.bucket.and_then(|b| self.buckets.get(b)) {
                    if let Some(duration) = bucket.get(msg.guild_id.unwrap()) {
                        let content = format!(
                            "Ratelimit reached. You may use this command in {:?}",
                            duration
                        );
                        let _ = context
                            .http
                            .create_message(msg.channel_id)
                            .content(content)
                            .unwrap()
                            .await;
                        return;
                    }
                }

                let args = Arguments::new(stream.rest());
                let args_count = Arguments::new(stream.rest()).count();
                if args_count < command.options.min_args {
                    let content = format!(
                        "```{}\n\n Expected atleast {} arguments, got only {}```",
                        msg.content, command.options.min_args, args_count
                    );
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content(content)
                        .unwrap()
                        .await;
                    return;
                }

                let res = (command.fun)(context, &msg, args).await;
                tracing::debug!(command = ?command.options.names[0], author = msg.author.id.0, "Command ran");
                match res {
                    Ok(()) => {
                        if let Some(bucket) =
                            command.options.bucket.and_then(|b| self.buckets.get(b))
                        {
                            bucket.take(msg.guild_id.unwrap());
                        }
                        if let Ok(metric) = context
                            .stats
                            .command_counts
                            .get_metric_with_label_values(&[&command.options.names[0]])
                        {
                            metric.inc();
                        }
                    }
                    Err(error) => self.handle_error(error, &context, &msg).await,
                }
            }
        }
    }

    pub async fn handle_event(&self, event: &Event, context: &Context) {
        if let Event::MessageCreate(msg) = event {
            self.dispatch(msg.0.clone(), context).await;
        }
    }

    fn run_checks(&self, context: &Context, msg: &Message, command: &Command) -> bool {
        if context.config.blocked_users.contains(&msg.author.id) {
            return false;
        }

        if context
            .config
            .blocked_guilds
            .contains(&msg.guild_id.unwrap())
        {
            return false;
        }

        if context.config.disabled_channels.contains(&msg.channel_id)
            && !command.options.names.contains(&"command-channel")
        {
            return false;
        }

        if context.config.owners.contains(&msg.author.id) {
            return true;
        }

        if let Some(guild) = context.cache.guild(msg.guild_id.unwrap()) {
            if context.config.blocked_users.contains(&guild.owner_id) {
                return false;
            }

            if msg.author.id.0 == guild.owner_id.0 {
                return true;
            }

            if let Some(member) = context.cache.member(guild.id, msg.author.id) {
                match command.options.perm_level {
                    RoLevel::Normal => return true,
                    RoLevel::Creator => {
                        if context.config.owners.contains(&msg.author.id) {
                            return true;
                        }
                    }
                    RoLevel::Council => {
                        if context.config.council.contains(&msg.author.id) {
                            return true;
                        }
                    }
                    RoLevel::Admin => {
                        if let Some(admin_role) = guild.admin_role {
                            if member.roles.contains(&admin_role) {
                                return true;
                            }
                        }
                        for role in member.roles.iter() {
                            if let Some(role) = context.cache.role(*role) {
                                if role.permissions.contains(Permissions::ADMINISTRATOR) {
                                    return true;
                                }
                            }
                        }
                    }
                    RoLevel::Trainer => return true,
                }
            }
        }
        false
    }

    async fn handle_error(&self, error: RoError, context: &Context, msg: &Message) {
        match error {
            RoError::Command(cmd_err) => match cmd_err {
                CommandError::Blacklist(reason) => {
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content(format!(
                            "User was found on the server blacklist. Reason: {}",
                            reason
                        ))
                        .unwrap()
                        .await;
                }
                CommandError::NicknameTooLong(nick) => {
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content(nick)
                        .unwrap()
                        .await;
                }
                CommandError::NoRoGuild => {
                    let _ = context.http.create_message(msg.channel_id)
                            .content("This server was not set up. Please ask the server owner to run `setup`").unwrap()
                            .await;
                }
                CommandError::ParseArgument(arg, param, param_type) => {
                    let idx = msg.content.find(&arg).unwrap();
                    let size = arg.len();
                    let content = format!(
                        "```{}\n{}{}\n\nExpected {} to be a {}```",
                        msg.content,
                        " ".repeat(idx),
                        "^".repeat(size),
                        param,
                        param_type
                    );
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content(content)
                        .unwrap()
                        .await;
                }
                CommandError::Timeout => {
                    let _ = context
                        .http
                        .create_message(msg.channel_id)
                        .content("Commmand cancelled. Please try again")
                        .unwrap()
                        .await;
                }
            },
            _ => {
                let _ = context.http.create_message(msg.channel_id)
                    .content("There was an error in executing this command. Please try again. If the issue persists, please contact the support server for more information").unwrap()
                    .await;
                let content = format!("```{}```", error);
                context.logger.log_debug(context, &content).await;
            }
        }
    }
}
