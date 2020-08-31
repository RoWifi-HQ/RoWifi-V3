use twilight::model::id::RoleId;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};
use twilight_mention::Mention;

pub async fn parse_username(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@!") {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if mention.starts_with("<@") {
        let len = mention.len() - 1;
        mention[2..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    } 
}

pub fn parse_role(mention: impl AsRef<str>) -> Option<u64> {
    let mention = mention.as_ref();

    if mention.len() < 4 {
        return None;
    }

    if mention.starts_with("<@&") && mention.ends_with('>') {
        let len = mention.len() - 1;
        mention[3..len].parse::<u64>().ok()
    } else if let Ok(r) = mention.parse::<u64>() {
        Some(r)
    } else {
        None
    }
}

pub trait EmbedExtensions {
    fn default_data(self) -> Self;
    fn update_log(self, added_roles: Vec<RoleId>, removed_roles: Vec<RoleId>, disc_nick: &str) -> Self;
}

impl EmbedExtensions for EmbedBuilder {
    #[inline]
    fn default_data(self) -> Self {
        self
            .timestamp(&chrono::Utc::now().to_rfc3339())
            .color(Color::Blue as u32).expect("Some shit occurred with the embed color")
            .footer(EmbedFooterBuilder::new("RoWifi").expect("Looks like the footer text screwed up"))
    }

    #[inline]
    fn update_log(self, added_roles: Vec<RoleId>, removed_roles: Vec<RoleId>, disc_nick: &str) -> Self {
        let mut added_str = added_roles.iter()
            .map(|a| format!("- {}\n", a.mention())).collect::<String>();
        let mut removed_str = removed_roles.iter()
            .map(|r| format!("- {}\n", r.mention())).collect::<String>();
        if added_str.is_empty() {
            added_str = "None".into();
        }
        if removed_str.is_empty() {
            removed_str = "None".into();
        }

        self
            .field(EmbedFieldBuilder::new("Nickname", disc_nick).unwrap())
            .field(EmbedFieldBuilder::new("Added Roles", added_str).unwrap())
            .field(EmbedFieldBuilder::new("Removed Roles", removed_str).unwrap())
    }
}

pub enum Color {
    Red = 0xE74C3C,
    Blue = 0x3498DB,
    DarkGreen = 0x1F8B4C
}