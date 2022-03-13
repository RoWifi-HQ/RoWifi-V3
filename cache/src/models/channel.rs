use rowifi_models::discord::{id::{Id, marker::ChannelMarker}, channel::{ChannelType, permission_overwrite::PermissionOverwrite}};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedChannel {
    pub id: Id<ChannelMarker>,
    pub name: Option<String>,
    pub kind: ChannelType,
    pub permission_overwrites: Vec<PermissionOverwrite>,
}