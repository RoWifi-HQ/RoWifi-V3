use std::num::ParseIntError;

use twilight_command_parser::Arguments;
use twilight_model::id::UserId;

#[derive(Debug)]
pub enum ArgumentError {
    MissingArgument,
    ParseError,
}

impl From<ParseIntError> for ArgumentError {
    fn from(err: ParseIntError) -> Self {
        ArgumentError::ParseError
    }
}

pub trait FromArgs {
    type Error;
    fn from_args(args: &mut Arguments<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

pub trait FromArg {
    type Error;
    fn from_arg(arg: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl FromArg for UserId {
    type Error = ParseIntError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        let id = u64::from_arg(arg)?;
        Ok(UserId(id))
    }
}

impl FromArg for u64 {
    type Error = ParseIntError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        arg.parse::<u64>()
    }
}
