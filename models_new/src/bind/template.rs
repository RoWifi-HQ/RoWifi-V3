use bytes::BytesMut;
use lazy_static::lazy_static;
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use regex::Regex;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::roblox::user::PartialUser as RobloxUser;
use crate::user::RoGuildUser;

lazy_static! {
    static ref TEMPLATE_REGEX: Regex = Regex::new(r"\{(.*?)\}").unwrap();
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Template(pub String);

impl Template {
    pub fn nickname(
        &self,
        roblox_user: &RobloxUser,
        user: &RoGuildUser,
        discord_username: &str,
    ) -> String {
        let roblox_id = user.roblox_id.to_string();
        let discord_id = user.discord_id.to_string();
        let display_name = roblox_user.display_name.clone().unwrap_or_default();

        let template_str = &self.0;
        let mut parts = vec![];

        let mut matches = TEMPLATE_REGEX
            .find_iter(template_str)
            .map(|m| (m.start(), m.end()))
            .peekable();
        let first = match matches.peek() {
            Some((start, _)) => *start,
            None => return template_str.clone(),
        };

        if first > 0 {
            parts.push(&template_str[0..first]);
        }

        let mut previous_end = first;
        for (start, end) in matches {
            if previous_end != start {
                parts.push(&template_str[previous_end..start]);
            }

            let arg = &template_str[start..end];
            let arg_name = &arg[1..arg.len() - 1];
            match arg_name {
                "roblox-username" => parts.push(&roblox_user.name),
                "roblox-id" => parts.push(&roblox_id),
                "discord-id" => parts.push(&discord_id),
                "discord-name" => parts.push(discord_username),
                "display-name" => parts.push(&display_name),
                _ => parts.push(arg),
            }

            previous_end = end;
        }

        if previous_end < template_str.len() {
            parts.push(&template_str[previous_end..]);
        }

        parts.join("")
    }

    pub fn has_slug(template_str: &str) -> bool {
        let matches = TEMPLATE_REGEX.find_iter(template_str);
        for m in matches {
            let match_str = m.as_str();
            let slug = &match_str[1..match_str.len() - 1];
            match slug {
                "roblox-username" | "roblox-id" | "discord-id" | "discord-name"
                | "display-name" => return true,
                _ => {}
            }
        }
        false
    }
}

impl Default for Template {
    fn default() -> Self {
        Self("{discord-name}".into())
    }
}

impl Display for Template {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl ToSql for Template {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        String::to_sql(&self.0, ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <String as ToSql>::accepts(ty)
    }

    to_sql_checked!();
}

impl<'a> FromSql<'a> for Template {
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(Template(String::from_sql(ty, raw)?))
    }

    fn accepts(ty: &Type) -> bool {
        <String as FromSql>::accepts(ty)
    }
}
