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
pub mod service;
pub mod prelude;

use std::{future::Future, pin::Pin, sync::Arc, task::{Context, Poll}};
use futures::future::{Either, Ready, ready};
use twilight_model::{gateway::event::Event, id::UserId};
use twilight_command_parser::Arguments;
use uwl::Stream;

use arguments::{FromArg, FromArgs, ArgumentError};
use command::Command;
use context::{BotContext, CommandContext};
use handler::{Handler, HandlerService};
use parser::PrefixType;
use error::RoError;
use service::Service;

pub type CommandResult = Result<(), RoError>;

pub struct Framework {
    bot: Arc<BotContext>,
    cmds: Vec<Command>
}

impl Framework {
    pub fn new(bot: Arc<BotContext>) -> Self
    {
        Self {
            bot,
            cmds: Vec::new()
        }
    }
}

impl Service<&Event> for Framework {
    type Response = ();
    type Error = RoError;
    type Future = Either<Ready<Result<(), Self::Error>>, Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>>;

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
                        if stream.rest().is_empty() && !self.bot.disabled_channels.contains(&msg.channel_id) {
                            let actual_prefix = self.bot.prefixes.get(&guild_id).map_or_else(|| self.bot.default_prefix.clone(), |p| p.to_string());
                            todo!("Respond to the user with the prefix");
                        }
                    }
                }

                if prefix.is_none() {
                    return Either::Left(ready(Ok(())));
                }
            },
            _ => {}
        }
        let fut = ready(Ok(()));
        Either::Left(fut)
    }
}

#[derive(Debug)]
pub struct UpdateArguments {
    pub user_id: UserId
}

impl FromArgs for UpdateArguments {
    type Error = String;
    fn from_args(args: &mut Arguments<'_>) -> Result<Self, Self::Error> {
        let user_id = match args.next() {
            Some(s) => UserId::from_arg(s).map_err(|_| String::from("Failed to parse integer"))?,
            None => return Err(String::from("Insufficient arguments"))
        };

        Ok(UpdateArguments {user_id})
    }
}

mod tests {
    use super::*;

    #[derive(Debug, FromArgs)]
    pub struct UpdateArguments2 {
        pub user_id: UserId
    }

    #[test]
    fn test() {
        let mut args = Arguments::new("311395138133950465");
        let ua = UpdateArguments2::from_args(&mut args);
        assert_eq!(ua.is_ok(), true);
    }
}