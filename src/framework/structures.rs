use futures::future::BoxFuture;
use std::{error::Error, fmt};
use twilight::{
    model::{guild::Permissions, channel::Message},
    command_parser::Arguments
};

use super::context::Context;

pub type CommandError = Box<dyn Error + Send + Sync>;
pub type CommandResult = std::result::Result<(), CommandError>;
pub type CommandFn = for<'fut> fn(&'fut Context, &'fut Message, Arguments) -> BoxFuture<'fut, CommandResult>;

pub struct Command {
    pub fun: CommandFn,
    pub options: &'static CommandOptions
}

#[derive(Debug, PartialEq)]
pub struct CommandOptions {
    pub bucket: Option<&'static str>,
    pub names: &'static [&'static str],
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub allowed_roles: &'static [&'static str],
    pub required_permissions: Permissions,
    pub hidden: bool,
    pub owners_only: bool,
    pub sub_commands: &'static [&'static Command]
}

pub type HelpCommandFn = for<'fut> fn(&'fut Context, &'fut Message, Arguments, &'fut HelpOptions, &'fut [&'static Command]);

pub struct HelpCommand {
    pub fun: HelpCommandFn,
    pub options: &'static HelpOptions
}

#[derive(Clone, Debug, PartialEq)]
pub struct HelpOptions {
    pub name: &'static str
}

pub struct Bucket {

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
            .field("options", &self.options)
            .finish()
    }
}

impl PartialEq for HelpCommand {
    #[inline]
    fn eq(&self, other: &HelpCommand) -> bool {
        (self.fun as usize == other.fun as usize) && (self.options == other.options)
    }
}