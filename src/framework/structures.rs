use futures::future::BoxFuture;
use std::{error::Error, sync::Arc};
use twilight::{
    model::{guild::Permissions, channel::Message},
    command_parser::Arguments
};

use super::context::Context;

#[derive(Debug, PartialEq)]
pub struct CommandGroup {
    pub name: &'static str,
    pub options: &'static GroupOptions
}

#[derive(Debug, PartialEq)]
pub struct GroupOptions {
    pub allowed_roles: &'static [&'static str],
    pub require_permissions: Permissions
}

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
    pub desc: &'static str,
    pub usage: &'static str,
    pub examples: &'static str,
    pub min_args: Option<u16>,
    pub max_args: Option<u16>,
    pub allowed_roles: &'static [&'static str],
    pub required_permissions: Permissions,
    pub hidden: bool,
    pub owners_only: bool,
    pub sub_commands: &'static [&'static Command]
}

pub struct Bucket {

}