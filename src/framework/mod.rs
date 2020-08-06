pub mod context;
mod map;
mod parser;
mod structures;

use std::{
    collections::{HashMap, HashSet}, 
    sync::Arc
};
use tokio::sync::Mutex;
use twilight::{
    gateway::Event,
    model::{
        channel::Message,
        id::{
            UserId, GuildId, ChannelId
        }
    }
};
use uwl::Stream;

use context::Context;
use map::CommandMap;
use structures::*;
use parser::{ParseError, Invoke};

#[derive(Default)]
pub struct Framework {
    commands: Vec<(&'static Command, CommandMap)>,
    help: Option<&'static HelpCommand>,
    config: Configuration
}

#[derive(Default)]
pub struct Configuration {
    pub blocked_guilds: HashSet<GuildId>,
    pub blocked_users: HashSet<UserId>,
    pub disabled_channels: HashSet<ChannelId>,
    pub prefixes: Arc<Mutex<HashMap<GuildId, String>>>,
    pub on_mention: String,
    pub default_prefix: String
}

impl Framework {
    async fn dispatch(&self, msg: Message, context: Context) {
        if msg.author.bot || msg.webhook_id.is_some() {
            return;
        }

        let mut stream = Stream::new(&msg.content);
        stream.take_while_char(|c| c.is_whitespace());

        let prefix = parser::find_prefix(&mut stream, &msg, &self.config).await;
        if prefix.is_some() && stream.rest().is_empty() {
            //print just the prefix
            return;
        }

        if prefix.is_none() {
            return;
        }

        let invocation = parser::command(&mut stream, &self.commands, &self.help.as_ref().map(|h| h.options.name)).await;
        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(_)) => {
                return;
            }
        };

        //check for perms

        match invoke {
            Invoke::Help => {

            },
            Invoke::Command{command} => {

            }
        }
    }

    pub async fn handle_event(&self, event: Event, context: Context) {
        match event {
            Event::MessageCreate(msg) => {
                self.dispatch(msg.0, context).await;
            },
            _ => {}
        }
    }
}