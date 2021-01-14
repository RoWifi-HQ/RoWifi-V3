use std::num::ParseIntError;
use twilight_model::{applications::CommandDataOption, id::UserId};

#[derive(Debug)]
pub struct Arguments {
    buf: Vec<String>,
    idx: usize,
}

#[derive(Debug)]
pub enum ArgumentError {
    MissingArgument,
    ParseError,
    BadArgument,
}

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
    type Error = ArgumentError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        let id = u64::from_arg(arg)?;
        Ok(UserId(id))
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        let id = u64::from_interaction(option)?;
        Ok(UserId(id))
    }
}

impl FromArg for u64 {
    type Error = ArgumentError;
    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        arg.parse::<u64>().map_err(|_| ArgumentError::ParseError)
    }

    fn from_interaction(option: &CommandDataOption) -> Result<Self, Self::Error> {
        match option {
            CommandDataOption::Integer { value, .. } => Ok(*value as u64),
            CommandDataOption::String { value, .. } => Ok(value.parse::<u64>()?),
            _ => Err(ArgumentError::BadArgument),
        }
    }
}

impl From<ParseIntError> for ArgumentError {
    fn from(err: ParseIntError) -> Self {
        ArgumentError::ParseError
    }
}
