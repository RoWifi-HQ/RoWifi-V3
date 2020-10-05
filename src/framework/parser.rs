use std::borrow::Cow;
use twilight_model::channel::Message;
use uwl::Stream;

use super::{Command, map::*};
use crate::models::configuration::Configuration;

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

pub fn find_prefix<'a>(stream: &mut Stream<'a>, msg: &Message, config: &Configuration) -> Option<Cow<'a, str>> {
    if let Some(id) = mention(stream, config) {
        stream.take_while_char(|c| c.is_whitespace());
        return Some(Cow::Borrowed(id));
    }

    if let Some(guild_id) = &msg.guild_id {
        if let Some(prefix) = config.prefixes.get(guild_id) {
            let peeked = stream.peek_for_char(prefix.chars().count());
            if prefix.value() == peeked {
                stream.increment(prefix.len());
                stream.take_while_char(|c| c.is_whitespace());
                return Some(Cow::Borrowed(peeked))
            } else {
                return None
            }
        }
    }

    let default_prefix = &config.default_prefix;
    let peeked = stream.peek_for_char(default_prefix.chars().count());
    if default_prefix == peeked {
        stream.increment(default_prefix.len());
        stream.take_while_char(|c| c.is_whitespace());
        return Some(Cow::Borrowed(peeked))
    }

    None
}

fn parse_command<'a>(stream: &'a mut Stream<'_>, map: &'a CommandMap) -> Result<&'static Command, ParseError> {
    let name = stream.peek_until_char(|c| c.is_whitespace());

    if let Some((cmd, map)) = map.get(&name.to_ascii_lowercase()) {
        stream.increment(name.len());

        stream.take_while_char(|c| c.is_whitespace());


        if map.is_empty() {
            return Ok(cmd);
        }

        return match parse_command(stream, &map) {
            Err(ParseError::UnrecognisedCommand(Some(_))) => Ok(cmd),
            res => res,
        };
    }

    Err(ParseError::UnrecognisedCommand(Some(name.to_string())))
}


pub fn command<'a>(stream: &mut Stream<'a>, commands: &[(&'static Command, CommandMap)], help: &Option<&'static str>) -> Result<Invoke, ParseError> {
    if let Some(help) = help {    
        let n = stream.peek_for_char(help.chars().count());
        if help.eq_ignore_ascii_case(n) {
            stream.increment(n.len());
            stream.take_while_char(|c| c.is_whitespace());
            return Ok(Invoke::Help);
        }
    }

    let mut last = Err(ParseError::UnrecognisedCommand(None));

    for (_command, map) in commands {
        let res = match parse_command(stream, map) {
            Ok(command) => return Ok(Invoke::Command { command }),
            Err(err) => Err(err)
        };
        last = res;
    }

    last
}

#[derive(Debug)]
pub enum Invoke {
    Command {
        command: &'static Command,
    },
    Help,
}

#[derive(Debug)]
pub enum ParseError {
    UnrecognisedCommand(Option<String>),
}