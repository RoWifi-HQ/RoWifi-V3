use rowifi_models::discord::id::GuildId;
use uwl::Stream;

use crate::context::BotContext;

#[derive(Debug)]
pub enum PrefixType<'p> {
    Mention,
    String(&'p str),
}

pub fn find_mention(stream: &mut Stream, on_mention: &str) -> bool {
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
        stream.take_while_char(char::is_whitespace);
        return Some(PrefixType::Mention);
    }

    if let Some(guild_id) = guild_id {
        if let Some(prefix) = bot.prefixes.get(&guild_id) {
            let peeked = stream.peek_for_char(prefix.chars().count());
            if prefix.value() == peeked {
                stream.increment(prefix.len());
                stream.take_while_char(char::is_whitespace);
                return Some(PrefixType::String(peeked));
            }
            return None;
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
