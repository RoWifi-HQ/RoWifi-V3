use futures::future::{BoxFuture, FutureExt};
use std::borrow::Cow;
use twilight::model::channel::Message;
use uwl::Stream;
use std::{collections::HashMap, sync::Arc};

use super::{Configuration, CommandGroup, HelpOptions, Command, map::*};

pub fn mention<'a>(stream: &mut Stream<'a>, config: &Configuration) -> Option<&'a str> {
    let on_mention = &config.on_mention;
    let start = stream.offset();
    if !stream.eat("<@") {
        return None;
    }
    stream.eat("!");

    let id = stream.take_while(|c| c.is_ascii_digit());
    if !stream.eat(">") {
        stream.set(start);
        return None;
    }

    if id == on_mention {
        Some(id)
    } else {
        stream.set(start);
        None
    }
}

pub async fn find_prefix<'a>(stream: &mut Stream<'a>, msg: &Message, config: &Configuration) -> Option<Cow<'a, str>> {
    if let Some(id) = mention(stream, config) {
        stream.take_while_char(|c| c.is_whitespace());
        return Some(Cow::Borrowed(id));
    }

    {
        let prefixes = config.prefixes.lock().await;
        if let Some(guild_id) = &msg.guild_id {
            if let Some(prefix) = prefixes.get(guild_id) {
                let peeked = stream.peek_for_char(prefix.chars().count());
                if prefix == peeked {
                    stream.increment(prefix.len());
                    return Some(Cow::Borrowed(peeked))
                }
            }
        }
    }

    let default_prefix = &config.default_prefix;
    let peeked = stream.peek_for_char(default_prefix.chars().count());
    if default_prefix == peeked {
        stream.increment(default_prefix.len());
        return Some(Cow::Borrowed(peeked))
    }

    None
}

fn parse_command<'a>(stream: &'a mut Stream<'_>, map: &'a CommandMap) -> BoxFuture<'a, Result<&'static Command, ParseError>> {
    async move {
        let name = stream.peek_until_char(|c| c.is_whitespace());

        if let Some((cmd, map)) = map.get(name) {
            stream.increment(name.len());

            stream.take_while_char(|c| c.is_whitespace());


            if map.is_empty() {
                return Ok(cmd);
            }

            return match parse_command(stream, &map).await {
                Err(ParseError::UnrecognisedCommand(Some(_))) => Ok(cmd),
                res => res,
            };
        }

        Err(ParseError::UnrecognisedCommand(Some(name.to_string())))
    }.boxed()
}

fn parse_group<'a>(stream: &'a mut Stream<'_>, map: &'a GroupMap) -> BoxFuture<'a, Result<(&'static CommandGroup, Arc<CommandMap>), ParseError>> {
    async move {
        let name = stream.peek_until_char(|c| c.is_whitespace());
        if let Some((group, map, commands)) = map.get(name) {
            stream.increment(name.len());

            stream.take_while_char(|c| c.is_whitespace());

            if map.is_empty() {
                return Ok((group, commands));
            }

            return match parse_group(stream, &map).await {
                Err(ParseError::UnrecognisedCommand(None)) => Ok((group, commands)),
                res => res,
            };
        }

        Err(ParseError::UnrecognisedCommand(None))
    }.boxed()
}

async fn handle_command<'a>(stream: &'a mut Stream<'_>, map: &'a CommandMap, group: &'static CommandGroup) -> Result<Invoke, ParseError> {
    match parse_command(stream, map).await {
        Ok(command) => Ok(Invoke::Command { group, command }),
        Err(err) => match group.options.default_command {
            Some(command) => Ok(Invoke::Command { group, command }),
            None => Err(err),
        },
    }
}


async fn handle_group<'a>(stream: &mut Stream<'_>, map: &'a GroupMap) -> Result<Invoke, ParseError> {
    match parse_group(stream, map).await {
        Ok((group, map)) => handle_command(stream, &map, group).await,
        Err(error) => Err(error),
    }
}

pub async fn command<'a>(stream: &mut Stream<'a>, msg: &Message, groups: &[(&'static CommandGroup, Map)], help: &HelpOptions) -> Result<Invoke, ParseError> {
    let n = stream.peek_for_char(help.name.chars().count());
    if help.name.eq_ignore_ascii_case(n) {
        stream.increment(n.len());
        stream.take_while_char(|c| c.is_whitespace());
        return Ok(Invoke::Help);
    }

    let mut is_prefixless = false;
    let mut last = Err(ParseError::UnrecognisedCommand(None));

    for (group, map) in groups {
        match map {
            Map::WithPrefixes(map) => {
                let res = handle_group(stream, map).await;
                if res.is_ok() {
                    return res;
                }

                if !is_prefixless {
                    last = res;
                }
            },
            Map::Prefixless(subgroups, commands) => {
                is_prefixless = true;
                let res = handle_group(stream,  subgroups).await;
                if res.is_ok() {
                    return res;
                }
                let res = handle_command(stream, commands, group).await;
                if res.is_ok() {
                    return res;
                }
                last = res;
            }
        }
    }

    last
}

#[derive(Debug)]
pub enum Invoke {
    Command {
        group: &'static CommandGroup,
        command: &'static Command,
    },
    Help,
}

#[derive(Debug)]
pub enum ParseError {
    UnrecognisedCommand(Option<String>),
}