use std::num::ParseIntError;
use twilight_model::id::UserId;

pub struct Arguments {
    buf: Vec<String>,
    idx: usize
}

#[derive(Debug)]
pub enum ArgumentError {
    MissingArgument,
    ParseError,
}

pub trait FromArgs {
    fn from_args(args: &mut Arguments) -> Result<Self, ArgumentError>
    where
        Self: Sized;
}

pub trait FromArg {
    type Error;
    fn from_arg(arg: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Arguments {
    pub fn new(buf: String) -> Self {
        let mut args = Vec::new();
        let mut start_idx = 0;
        let mut quoted = false;
        let mut started = false;

        while let Some((i, ch)) = buf.char_indices().next() {
            if quoted {
                if ch == '"' {
                    let v = buf[start_idx..i].trim();
                    args.push(v.to_string());
                }
            } else if ch == ' ' {
                if started {
                    let v = buf[start_idx..i].trim();
                    args.push(v.to_string());
                } else {
                    start_idx = i;
                    started = true;
                    continue;
                }
            } else if ch == '"' {
                start_idx = i + 1;
                quoted = true;
            }
        }

        Self {
            buf: args,
            idx: 0
        }
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
    T: FromArg
{
    type Error = <T as FromArg>::Error;

    fn from_arg(arg: &str) -> Result<Self, Self::Error> {
        Ok(match T::from_arg(arg) {
            Ok(arg) => Some(arg),
            Err(_) => None
        })
    }
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

impl From<ParseIntError> for ArgumentError {
    fn from(err: ParseIntError) -> Self {
        ArgumentError::ParseError
    }
}
