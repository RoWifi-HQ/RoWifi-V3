use twilight::model::{
    guild::Permissions,
    id::{RoleId, GuildId},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedRole {
    pub id: RoleId,
    pub guild_id: GuildId,
    pub name: String,
    pub position: i64,
    pub permissions: Permissions
}