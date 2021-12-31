use rowifi_models::{discord::{
    guild::{Member, PartialMember},
    user::User,
}, id::RoleId};
use std::sync::Arc;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedMember {
    pub roles: Vec<RoleId>,
    pub nick: Option<String>,
    pub user: Arc<User>,
    pub pending: bool,
}

impl PartialEq<Member> for CachedMember {
    fn eq(&self, other: &Member) -> bool {
        (&self.roles.iter().map(|r| r.0).collect(), &self.nick) == (&other.roles, &other.nick)
    }
}

impl PartialEq<&PartialMember> for CachedMember {
    fn eq(&self, other: &&PartialMember) -> bool {
        (&self.nick, &self.roles.iter().map(|r| r.0).collect()) == (&other.nick, &other.roles)
    }
}
