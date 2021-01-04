use twilight_model::id::GuildId;
use uwl::Stream;

use crate::{command::Command, context::BotContext};

pub enum PrefixType<'p> {
    Mention,
    String(&'p str),
}

pub fn find_mention<'a>(stream: &mut Stream, on_mention: &str) -> bool {
    let start = stream.offset();
    if !stream.eat("<@") {
        return false;
    }
    stream.eat("!");

    let id = stream.take_while(|c| c.is_ascii_digit());
    if !stream.eat(">") {
        stream.set(start);
        return false;
    }

    if id == on_mention {
        true
    } else {
        stream.set(start);
        false
    }
}

pub fn find_prefix<'a>(
    stream: &mut Stream<'a>,
    bot: &BotContext,
    guild_id: Option<GuildId>,
) -> Option<PrefixType<'a>> {
    if find_mention(stream, bot.on_mention.as_ref()) {
        return Some(PrefixType::Mention);
    }

    if let Some(guild_id) = guild_id {
        if let Some(prefix) = bot.prefixes.get(&guild_id) {
            let peeked = stream.peek_for_char(prefix.chars().count());
            if prefix.value() == peeked {
                stream.increment(prefix.len());
                stream.take_while_char(char::is_whitespace);
                return Some(PrefixType::String(peeked));
            } else {
                return None;
            }
        }
    }

    let default_prefix = &bot.default_prefix;
    let peeked = stream.peek_for_char(default_prefix.chars().count());
    if default_prefix.eq(peeked) {
        stream.increment(default_prefix.len());
        stream.take_while_char(char::is_whitespace);
        return Some(PrefixType::String(peeked));
    }

    None
}

fn parse_command<'a>(stream: &mut Stream<'a>, command: &'a Command) -> Option<&'a Command> {
    let sub_name = stream.peek_until_char(char::is_whitespace);
    if let Some(sub_cmd) = command.sub_commands.get(&sub_name.to_ascii_lowercase()) {
        stream.increment(sub_name.len());
        stream.take_while_char(char::is_whitespace);
        if sub_cmd.sub_commands.is_empty() {
            return Some(sub_cmd);
        }

        return parse_command(stream, sub_cmd);
    }
    None
}

pub fn find_command<'a>(stream: &mut Stream<'a>, commands: &'a [Command]) -> Option<&'a Command> {
    //TODO: Do the help command
    let name = stream
        .peek_until_char(char::is_whitespace)
        .to_ascii_lowercase();
    for cmd in commands {
        if cmd.names.contains(&name.as_ref()) {
            return match parse_command(stream, cmd) {
                Some(c) => Some(c),
                None => {
                    stream.increment(name.len());
                    Some(cmd)
                },
            };
        }
    }

    None
}
