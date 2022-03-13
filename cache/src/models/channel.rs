use rowifi_models::discord::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    id::{marker::ChannelMarker, Id},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedChannel {
    pub id: Id<ChannelMarker>,
    pub name: Option<String>,
    pub kind: ChannelType,
    pub permission_overwrites: Vec<PermissionOverwrite>,
}
