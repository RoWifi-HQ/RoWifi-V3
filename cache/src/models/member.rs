use std::sync::Arc;
use twilight_model::{
    guild::{Member, PartialMember},
    id::RoleId,
    user::User,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CachedMember {
    pub roles: Vec<RoleId>,
    pub nick: Option<String>,
    pub user: Arc<User>,
    pub pending: bool,
}

impl PartialEq<Member> for CachedMember {
    fn eq(&self, other: &Member) -> bool {
        (&self.roles, &self.nick) == (&other.roles, &other.nick)
    }
}

impl PartialEq<&PartialMember> for CachedMember {
    fn eq(&self, other: &&PartialMember) -> bool {
        (&self.nick, &self.roles) == (&other.nick, &other.roles)
    }
}
