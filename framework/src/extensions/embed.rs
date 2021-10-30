use rowifi_models::discord::{datetime::Timestamp, id::RoleId};
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder};

use crate::utils::Color;

pub trait EmbedExtensions {
    fn default_data(self) -> Self;
    fn update_log(self, added_roles: &[RoleId], removed_roles: &[RoleId], disc_nick: &str) -> Self;
}

impl EmbedExtensions for EmbedBuilder {
    fn default_data(self) -> Self {
        self.timestamp(Timestamp::from_secs(chrono::Utc::now().timestamp() as u64).unwrap())
            .color(Color::Blue as u32)
            .footer(EmbedFooterBuilder::new("RoWifi"))
    }

    fn update_log(self, added_roles: &[RoleId], removed_roles: &[RoleId], disc_nick: &str) -> Self {
        let mut added_str = added_roles
            .iter()
            .map(|a| format!("- <@&{}>\n", a.0))
            .collect::<String>();
        let mut removed_str = removed_roles
            .iter()
            .map(|r| format!("- <@&{}>\n", r.0))
            .collect::<String>();
        if added_str.is_empty() {
            added_str = "None".into();
        }
        if removed_str.is_empty() {
            removed_str = "None".into();
        }

        self.field(EmbedFieldBuilder::new("Nickname", disc_nick))
            .field(EmbedFieldBuilder::new("Added Roles", added_str))
            .field(EmbedFieldBuilder::new("Removed Roles", removed_str))
    }
}
