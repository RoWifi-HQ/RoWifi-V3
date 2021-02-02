#![allow(dead_code)]
#![allow(unused_variables)]

pub mod arguments;
pub mod bucket;
pub mod command;
pub mod context;
pub mod error;
pub mod handler;
mod parser;
pub mod prelude;
pub mod utils;

use futures::future::{ready, Either, Ready};
use rowifi_cache::{CachedGuild, CachedMember};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use twilight_model::{
    applications::interaction::Interaction,
    gateway::event::Event,
    guild::Permissions,
    id::{ChannelId, GuildId, UserId},
};
use uwl::Stream;

use arguments::{ArgumentError, Arguments, FromArg, FromArgs};
use command::{Command, ServiceRequest};
use context::{BotContext, CommandContext};
use error::RoError;
use parser::PrefixType;
use utils::RoLevel;

pub type CommandResult = Result<(), RoError>;
pub use framework_derive::FromArgs;

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

    pub fn configure<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut Vec<Command>),
    {
        func(&mut self.cmds);
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

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
                let mut cmd_str = Arguments::new(content);

                let command = if let Some(arg) = cmd_str.next() {
                    self.cmds.iter_mut().find(|c| c.names.contains(&arg))
                } else {
                    None
                };

                let command = match command {
                    Some(c) => c,
                    None => return Either::Left(ready(Ok(()))),
                };

                if !run_checks(
                    &self.bot,
                    command,
                    msg.guild_id,
                    msg.channel_id,
                    msg.author.id,
                ) {
                    return Either::Left(ready(Ok(())));
                }

                let ctx = CommandContext {
                    bot: self.bot.clone(),
                    channel_id: msg.channel_id,
                    guild_id: msg.guild_id,
                    author_id: msg.author.id,
                };

                let request = ServiceRequest::Message(cmd_str);
                let cmd_fut = command.call((ctx, request));
                let fut = async move {
                    //A global before handler
                    //Bucket handler
                    cmd_fut.await
                    //Add the metrics here
                    //A global after handler (includes the error handler)
                };
                return Either::Right(Box::pin(fut));
            }
            Event::InteractionCreate(interaction) => {
                if let Interaction::ApplicationCommand(top_command) = &interaction.0 {
                    let command_options = &top_command.command_data.options;
                    let command = self
                        .cmds
                        .iter_mut()
                        .find(|c| c.names.contains(&top_command.command_data.name.as_str()));
                    println!("{:?}", command);
                    let command = match command {
                        Some(c) => c,
                        None => return Either::Left(ready(Ok(()))),
                    };

                    if !run_checks(
                        &self.bot,
                        command,
                        Some(top_command.guild_id),
                        top_command.channel_id,
                        top_command.member.user.clone().unwrap().id,
                    ) {
                        return Either::Left(ready(Ok(())));
                    }

                    let ctx = CommandContext {
                        bot: self.bot.clone(),
                        channel_id: top_command.channel_id,
                        guild_id: Some(top_command.guild_id),
                        author_id: top_command.member.user.clone().unwrap().id,
                    };

                    let request = ServiceRequest::Interaction(command_options.to_owned());
                    let cmd_fut = command.call((ctx, request));
                    let fut = async move {
                        //A global before handler
                        //Bucket handler
                        cmd_fut.await
                        //Add the metrics here
                        //A global after handler (includes the error handler)
                    };
                    return Either::Right(Box::pin(fut));
                }
            }
            _ => {}
        }
        let fut = ready(Ok(()));
        Either::Left(fut)
    }
}

fn run_checks(
    bot: &BotContext,
    cmd: &Command,
    guild_id: Option<GuildId>,
    channel_id: ChannelId,
    author: UserId,
) -> bool {
    if bot.disabled_channels.contains(&channel_id) && cmd.names.contains(&"command-channel") {
        return false;
    }

    if bot.owners.contains(&author) {
        return true;
    }

    if let Some(guild_id) = guild_id {
        if let Some(guild) = bot.cache.guild(guild_id) {
            if let Some(member) = bot.cache.member(guild_id, author) {
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
        #[arg(help = "User to update")]
        pub user_id: UserId,
        pub priority: u64,
    }

    pub async fn update(_ctx: CommandContext, _args: UpdateArguments2) -> Result<(), RoError> {
        Ok(())
    }

    #[test]
    pub fn test_update() {
        let mut args = Arguments::new("12345".into());
        assert_eq!(UpdateArguments2::from_args(&mut args).is_ok(), true);
    }

    #[test]
    pub fn test_builder() {
        let command = Command::builder()
            .names(&["update2"])
            .service(Box::new(handler::CommandHandler::new(update)));
    }
}
