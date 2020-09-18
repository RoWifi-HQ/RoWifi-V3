pub mod context;
mod configuration;
mod map;
pub mod parser;
pub mod prelude;
pub mod structures;
pub mod utils;

use twilight_gateway::Event;
use twilight_model::{
    channel::Message,
    guild::Permissions
};
use twilight_command_parser::Arguments;
use uwl::Stream;
use crate::utils::error::{RoError, CommandError};

use context::Context;
use configuration::Configuration;
pub use map::CommandMap;
use structures::*;
use parser::{ParseError, Invoke};

#[derive(Default)]
pub struct Framework {
    commands: Vec<(&'static Command, CommandMap)>,
    help: Option<&'static HelpCommand>,
    config: Configuration
}

impl Framework {
    pub fn configure<F>(mut self, f: F) -> Self where F: FnOnce(&mut Configuration) -> &mut Configuration {
        f(&mut self.config);
        self
    }

    pub fn command(mut self, command: &'static Command) -> Self {
        let map = CommandMap::new(&[command]);
        self.commands.push((command, map));
        self
    }

    pub fn help(mut self, help: &'static HelpCommand) -> Self {
        self.help = Some(help);
        self
    }

    async fn dispatch(&self, msg: Message, mut context: Context) {
        if msg.author.bot || msg.webhook_id.is_some() || msg.guild_id.is_none() || msg.content.is_empty() {
            return;
        }

        let mut stream = Stream::new(&msg.content);
        stream.take_while_char(|c| c.is_whitespace());

        let prefix = parser::find_prefix(&mut stream, &msg, &self.config).await;
        if prefix.is_some() && stream.rest().is_empty() {
            let command_prefix = self.config.prefixes.get(&msg.guild_id.unwrap()).map(|g| g.to_string()).unwrap_or_else(|| self.config.default_prefix.to_string());
            let _ = context.http.create_message(msg.channel_id).content(format!("The prefix of this server is {}", command_prefix)).unwrap().await;
            return;
        }

        if prefix.is_none() {
            return;
        }

        let invocation = parser::command(&mut stream, &self.commands, &self.help.as_ref().map(|h| h.name)).await;
        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(_)) => {
                return;
            }
        };

        match invoke {
            Invoke::Help => {
                let args = Arguments::new(stream.rest());
                if let Some(help) = self.help {
                    let _res = (help.fun)(&mut context, &msg, args, &self.commands).await;
                }
            },
            Invoke::Command{command} => {
                // if !self.run_checks(&context, &msg, command).await {
                //     return;
                // }
                let args = Arguments::new(stream.rest());

                let res = (command.fun)(&mut context, &msg, args).await;

                match res {
                    Ok(()) => {},
                    Err(error) => self.handle_error(error, &context, &msg).await
                }
            }
        }
    }

    pub async fn handle_event(&self, event: Event, context: Context) {
        if let Event::MessageCreate(msg) = event {
            self.dispatch(msg.0, context).await;
        }
    }

    async fn run_checks(&self, context: &Context, msg: &Message, command: &Command) -> bool {
        if self.config.blocked_users.contains(&msg.author.id) {
            return false;
        }

        if self.config.owners.contains(&msg.author.id) {
            return true;
        }

        //Check for disabled channels (Implement after database cache)
        //Bucketing mechanism

        if let Some(guild) = context.cache.guild(msg.guild_id.unwrap()) {
            if self.config.blocked_users.contains(&guild.owner_id) {
                return false;
            }

            if msg.author.id.0 == guild.owner_id.0 {
                return true;
            }

            if let Some(member) = context.cache.member(guild.id, msg.author.id) {
                for role in member.roles.iter() {
                    if let Some(role) = context.cache.role(*role) {
                        if role.permissions.contains(Permissions::ADMINISTRATOR) {
                            return true;
                        }
                        if role.permissions.contains(command.options.required_permissions) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    async fn handle_error(&self, error: RoError, context: &Context, msg: &Message) {
        match error {
            RoError::Command(cmd_err) => {
                match cmd_err {
                    CommandError::Blacklist(reason) => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content(format!("User was found on the server blacklist. Reason: {}", reason)).unwrap()
                            .await;
                    },
                    CommandError::NicknameTooLong(nick) => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content(format!("The supposed nickname {} was found to be longer than 32 characters", nick)).unwrap()
                            .await;
                    },
                    CommandError::NoRoGuild => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content("This server was not set up. Please ask the server owner to run `setup`").unwrap()
                            .await;
                    }
                    CommandError::ParseArgument(arg, param, param_type) => {
                        let idx = msg.content.find(&arg).unwrap();
                        let size = arg.len();
                        let content = format!("```{}\n{}{}\n\nExpected {} to be a {}```", 
                            msg.content, " ".repeat(idx), "^".repeat(size), param, param_type
                        );
                        let _ = context.http.create_message(msg.channel_id)
                            .content(content).unwrap().await;
                    },
                    CommandError::Timeout => {
                        let _ = context.http.create_message(msg.channel_id)
                            .content("Timeout reached. Please try again").unwrap().await;
                    }
                }
            },
            _ => {
                let _ = context.http.create_message(msg.channel_id)
                    .content("There was an error in executing this command. Please try again. If the issue persists, please contact the support server for more information").unwrap()
                    .await;
            }
        }
    }
}