use rowifi_models::{bind::AssetType, guild::BlacklistActionType};
use std::{num::ParseIntError, str::FromStr};
use twilight_model::{
    applications::interaction::CommandDataOption,
    id::{RoleId, UserId},
};

use crate::utils::{parse_role, parse_username};

#[derive(Debug, Clone)]
pub struct Arguments {
    buf: Vec<String>,
    idx: usize,
}

#[derive(Debug)]
pub enum ArgumentError {
    MissingArgument {
        usage: (&'static str, &'static str),
        name: &'static str,
    },
    ParseError {
        expected: &'static str,
        usage: (&'static str, &'static str),
        name: &'static str,
    },
    BadArgument,
}

pub struct ParseError(pub &'static str);

pub trait FromArgs {
    fn from_args(args: &mut Arguments) -> Result<Self, ArgumentError>
    where
        Self: Sized;

    fn from_interaction(options: &[CommandDataOption]) -> Result<Self, ArgumentError>
    where
        Self: Sized;

    fn generate_help() -> (&'static str, &'static str);
}

pub trait FromArg {
    type Error;
    fn from_arg(arg: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Arguments {
    pub fn new(buf: String) -> Self {
        let mut args = Vec::new();
        let mut start_idx = 0;
        let mut quoted = false;
        let mut started = false;
        let mut idxs = buf.char_indices();

        while let Some((i, ch)) = idxs.next() {
            if quoted {
                if ch == '"' {
                    let v = buf[start_idx..i].trim();
                    args.push(v.to_string());
                    start_idx = i + 1;
                }
            } else if ch == ' ' {
                if started {
                    let v = buf[start_idx..i].trim();
                    args.push(v.to_string());
                    start_idx = i + 1;
                } else {
                    start_idx = i;
                    started = true;
                    continue;
                }
            } else if ch == '"' {
                start_idx = i + 1;
                quoted = true;
            }
            started = true;
        }

        match buf.get(start_idx..) {
            Some("") | None => {}
            Some(s) => args.push(s.to_string()),
        }

        Self { buf: args, idx: 0 }
    }

    pub fn next(&mut self) -> Option<&str> {
        let res = self.buf.get(self.idx);
        self.idx += 1;
        res.map(|s| s.as_str())
    }

    pub fn back(&mut self) {
        self.idx -= 1;
    }

    pub fn rest(&self) -> Option<String> {
        let res = self.buf.get(self.idx..);
        res.map(|s| s.join(" "))
    }
}

impl<T> FromArg for Option<T>
where
    T: FromArg,
{
    type Error = <T as FromArg>::Error;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        Ok(match T::from_arg(arg) {
            Ok(arg) => Some(arg),
            Err(_) => None,
        })
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        Ok(match T::from_interaction(option) {
            Ok(arg) => Some(arg),
            Err(_) => None,
        })
    }
}

impl FromArg for UserId {
    type Error = ParseError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match parse_username(arg) {
            Some(id) => Ok(UserId(id)),
            None => Err(ParseError("an User")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(UserId(*value as u64)),
            CommandDataOption::String { value, .. } => Self::from_arg(value),
            _ => unreachable!("UserId unreached"),
        }
    }
}

impl FromArg for RoleId {
    type Error = ParseError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match parse_role(arg) {
            Some(id) => Ok(RoleId(id)),
            None => Err(ParseError("a Role")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(RoleId(*value as u64)),
            CommandDataOption::String { value, .. } => Self::from_arg(value),
            _ => unreachable!("RoleId unreached"),
        }
    }
}

impl FromArg for u64 {
    type Error = ParseError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        arg.parse::<u64>().map_err(|_| ParseError("a number"))
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(*value as u64),
            CommandDataOption::String { value, .. } => Ok(value.parse::<u64>()?),
            _ => unreachable!("u64 unreached"),
        }
    }
}

impl FromArg for i64 {
    type Error = ParseError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        arg.parse::<i64>().map_err(|_| ParseError("a number"))
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(*value as i64),
            CommandDataOption::String { value, .. } => Ok(value.parse::<i64>()?),
            _ => unreachable!("i64 unreached"),
        }
    }
}

impl FromArg for String {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        Ok(arg.to_owned())
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(value.to_string()),
            CommandDataOption::String { value, .. } => Ok(value.to_owned()),
            _ => unreachable!("String unreached"),
        }
    }
}

impl FromArg for AssetType {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match AssetType::from_str(arg) {
            Ok(a) => Ok(a),
            Err(_) => Err(ParseError("one of `Asset` `Badge` `Gamepass`")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("AssetType unreached"),
        };

        AssetType::from_arg(&arg)
    }
}

impl FromArg for BlacklistActionType {
    type Error = ParseError;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        match BlacklistActionType::from_str(arg) {
            Ok(a) => Ok(a),
            Err(_) => Err(ParseError("one of `None` `Kick` `Ban`")),
        }
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let arg = match option {
            CommandDataOption::String { value, .. } => value.to_string(),
            CommandDataOption::Integer { value, .. } => value.to_string(),
            _ => unreachable!("BlacklistActionType unreached"),
        };

        Self::from_arg(&arg)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(_err: ParseIntError) -> Self {
        ParseError("a number")
    }
}
