use futures::future::BoxFuture;
use std::{
    fmt,
    time::{Duration, Instant},
};
use transient_dashmap::TransientDashMap;
use twilight_command_parser::Arguments;
use twilight_model::{channel::Message, id::GuildId};

use super::{Context, RoError, map::CommandMap};

pub type CommandError = RoError;
pub type CommandResult = std::result::Result<(), CommandError>;
pub type CommandFn =
    for<'fut> fn(&'fut Context, &'fut Message, Arguments<'fut>) -> BoxFuture<'fut, CommandResult>;

pub struct Command {
    pub fun: CommandFn,
    pub options: &'static CommandOptions,
}

#[derive(Debug, PartialEq)]
pub struct CommandOptions {
    pub perm_level: RoLevel,
    pub bucket: Option<&'static str>,
    pub names: &'static [&'static str],
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub min_args: usize,
    pub hidden: bool,
    pub sub_commands: &'static [&'static Command],
    pub group: Option<&'static str>,
}

pub type HelpCommandFn = for<'fut> fn(
    &'fut Context,
    &'fut Message,
    Arguments<'fut>,
    &'fut [(&'static Command, CommandMap)],
) -> BoxFuture<'fut, CommandResult>;

pub struct HelpCommand {
    pub fun: HelpCommandFn,
    pub name: &'static str,
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
#[repr(i8)]
pub enum RoLevel {
    Creator = 4,
    Admin = 3,
    Trainer = 2,
    Council = 1,
    Normal = 0,
}

pub struct Bucket {
    pub time: Duration,
    pub guilds: TransientDashMap<GuildId, u64>,
    pub calls: u64,
}

impl Bucket {
    pub fn take(&self, guild_id: GuildId) -> Option<Duration> {
        let (new_remaining, expiration) = match self.guilds.get(&guild_id) {
            Some(g) => {
                let remaining = g.object;
                if remaining == 0 {
                    return g.expiration.checked_duration_since(Instant::now());
                }
                (remaining - 1, g.expiration)
            }
            None => {
                self.guilds.insert(guild_id, self.calls - 1);
                return None;
            }
        };
        self.guilds
            .insert_with_expiration(guild_id, new_remaining, expiration);
        None
    }

    pub fn get(&self, guild_id: GuildId) -> Option<Duration> {
        match self.guilds.get(&guild_id) {
            Some(g) => {
                if g.object == 0 {
                    return g.expiration.checked_duration_since(Instant::now());
                }
                None
            }
            None => None,
        }
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("options", &self.options)
            .finish()
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Command) -> bool {
        (self.fun as usize == other.fun as usize) && (self.options == other.options)
    }
}

impl fmt::Debug for HelpCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelpCommand")
            .field("fun", &"<function>")
            .finish()
    }
}
