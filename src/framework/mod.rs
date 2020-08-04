mod context;
mod map;
mod parser;
mod structures;

use std::{
    collections::{HashMap, HashSet}, 
    sync::Arc
};
use tokio::sync::{Mutex, RwLock};
use typemap_rev::TypeMap;
use twilight::{
    command_parser::Arguments,
    model::{
        channel::Message,
        id::{
            UserId, GuildId, ChannelId
        }
    }
};
use uwl::Stream;

use map::Map;
use structures::*;
use parser::{ParseError, Invoke};

pub struct Framework {
    data: Arc<RwLock<TypeMap>>,
    groups: Vec<(&'static CommandGroup, Map)>,
    buckets: Mutex<HashMap<String, Bucket>>,
    help: &'static HelpCommand,
    config: Configuration
}

pub struct Configuration {
    pub blocked_guilds: HashSet<GuildId>,
    pub blocked_users: HashSet<UserId>,
    pub disabled_channels: HashSet<ChannelId>,
    pub prefixes: Arc<Mutex<HashMap<GuildId, String>>>,
    pub on_mention: String,
    pub default_prefix: String
}

impl Framework {
    async fn dispatch(&self, msg: Message) {
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

        //create context

        let invocation = parser::command(&mut stream, &msg, &self.groups, &self.help.options).await;
        let invoke = match invocation {
            Ok(i) => i,
            Err(ParseError::UnrecognisedCommand(_)) => {
                return;
            }
        };

        match invoke {
            Invoke::Help => {

            },
            Invoke::Command {command, group} => {
                let mut args = Arguments::new(stream.rest());
                //check for permissions blah blah
                //let res = (command.fun)(&mut ctx, &msg, args).await;
            }
        }
    }
}