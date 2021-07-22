mod embed;
mod standby;

pub use embed::EmbedExtensions;
pub use standby::StandbyExtensions;

use twilight_model::{application::interaction::MessageComponentInteraction, id::UserId};

pub trait MessageComponentExtensions {
    fn author_id(&self) -> Option<UserId>;
}

impl MessageComponentExtensions for MessageComponentInteraction {
    fn author_id(&self) -> Option<UserId> {
        self.member.as_ref().map_or_else(
            || self.user.as_ref().map(|user| user.id),
            |member| member.user.as_ref().map(|u| u.id),
        )
    }
}
