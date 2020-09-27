use futures::future::BoxFuture;
use std::fmt;
use twilight_model::{guild::Permissions, channel::Message};
use twilight_command_parser::Arguments;
use crate::utils::error::RoError;

use super::context::Context;
use super::map::CommandMap;

pub type CommandError = RoError;
pub type CommandResult = std::result::Result<(), CommandError>;
pub type CommandFn = for<'fut> fn(&'fut Context, &'fut Message, Arguments<'fut>) -> BoxFuture<'fut, CommandResult>;

pub struct Command {
    pub fun: CommandFn,
    pub options: &'static CommandOptions
}

#[derive(Debug, PartialEq)]
pub struct CommandOptions {
    pub perm_level: RoLevel,
    pub bucket: Option<&'static str>,
    pub names: &'static [&'static str],
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub required_permissions: Permissions,
    pub min_args: usize,
    pub hidden: bool,
    pub sub_commands: &'static [&'static Command],
    pub group: Option<&'static str>
}

pub type HelpCommandFn = for<'fut> fn(&'fut Context, &'fut Message, Arguments<'fut>, &'fut [(&'static Command, CommandMap)]) -> BoxFuture<'fut, CommandResult>;

pub struct HelpCommand {
    pub fun: HelpCommandFn,
    pub name: &'static str
}

#[derive(Debug, PartialEq)]
#[repr(i8)]
pub enum RoLevel {
    Creator = 3, 
    Admin = 2,
    Trainer = 1,
    Normal = 0 
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
            .finish()
    }
}