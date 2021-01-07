#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate framework_derive;

pub mod arguments;
pub mod command;
pub mod context;
pub mod error;
pub mod handler;
mod parser;
pub mod prelude;
pub mod service;

use futures::future::{ready, Either, Ready};
use rowifi_cache::{CachedGuild, CachedMember};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use twilight_model::{applications::InteractionData, channel::Message, gateway::event::Event, guild::Permissions, id::UserId};
use uwl::Stream;

use arguments::{ArgumentError, FromArg, FromArgs, Arguments};
use command::{Command, RoLevel};
use context::{BotContext, CommandContext};
use error::RoError;
use handler::{Handler, HandlerService};
use parser::PrefixType;
use service::Service;

pub type CommandResult = Result<(), RoError>;

pub struct Framework {
    bot: BotContext,
    cmds: Vec<Command>,
}

impl Framework {
    pub fn new(bot: BotContext) -> Self {
        Self {
            bot,
            cmds: Vec::new(),
        }
    }

    pub fn command(mut self, cmd: Command) -> Self {
        self.cmds.push(cmd);
        self
    }
}

impl Service<&Event> for Framework {
    type Response = ();
    type Error = RoError;
    type Future = Either<
        Ready<Result<(), Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>,
    >;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: &Event) -> Self::Future {
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
                            todo!("Respond to the user with the prefix");
                        }
                    }
                }

                if prefix.is_none() {
                    return Either::Left(ready(Ok(())));
                }

                let content = stream.rest().to_string();
                let mut command: Option<&Command> = None;
                let mut cmd_str = Arguments::new(content);
                while let Some(arg) = cmd_str.next() {
                    println!("{:?}", arg);
                    if let Some(c) = command {
                        if let Some(sub_cmd) = c.sub_commands.get(&arg.to_ascii_lowercase()) {
                            command = Some(sub_cmd);
                            if sub_cmd.sub_commands.is_empty() {
                                cmd_str.back();
                                break;
                            }
                        } else {
                            cmd_str.back();
                            break;
                        }
                    }
                    else {
                        for cmd in &self.cmds {
                            if cmd.names.contains(&arg) {
                                command = Some(cmd);
                            }
                        }
                    }
                }

                let command = match command {
                    Some(c) => c,
                    None => return Either::Left(ready(Ok(()))),
                };
                

                if !run_checks(&self.bot, command, &msg) {
                    return Either::Left(ready(Ok(())));
                }

                let ctx = CommandContext {
                    bot: self.bot.clone(),
                    msg: msg.0.clone()
                };

                let cmd_fut = command.call((ctx, cmd_str));
                let fut = async move {
                    //A global before handler
                    //Bucket handler
                    cmd_fut.await
                    //Add the metrics here
                    //A global after handler (includes the error handler)
                };
                return Either::Right(Box::pin(fut));
            },
            Event::InteractionCreate(interaction) => {
                if let InteractionData::ApplicationCommand(command) = &interaction.data {
                    
                }
            }
            _ => {}
        }
        let fut = ready(Ok(()));
        Either::Left(fut)
    }
}

fn run_checks(bot: &BotContext, cmd: &Command, msg: &Message) -> bool {
    if bot.disabled_channels.contains(&msg.channel_id) && cmd.names.contains(&"command-channel") {
        return false;
    }

    if bot.owners.contains(&msg.author.id) {
        return true;
    }

    if let Some(guild_id) = msg.guild_id {
        if let Some(guild) = bot.cache.guild(guild_id) {
            if let Some(member) = bot.cache.member(guild_id, msg.author.id) {
                return cmd.options.level <= get_perm_level(bot, &guild, &member);
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

    RoLevel::Normal
}

mod tests {
    use super::*;

    #[derive(Debug, FromArgs)]
    pub struct UpdateArguments2 {
        pub user_id: UserId,
    }

    #[test]
    fn test() {
        let mut args = Arguments::new("311395138133950465".into());
        let ua = UpdateArguments2::from_args(&mut args);
        assert_eq!(ua.is_ok(), true);
    }
}
