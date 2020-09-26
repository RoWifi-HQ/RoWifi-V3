use dashmap::{mapref::entry::Entry, DashMap, DashSet};
use std::{
    collections::HashSet,
    hash::Hash,
    sync::{Arc, Mutex},
};
use twilight_model::{
    channel::GuildChannel,
    guild::{Guild, Member, Role},
    id::{ChannelId, GuildId, RoleId, UserId},
    user::{CurrentUser, User},
};

mod event;
mod models;
use event::UpdateCache;
pub use models::{guild::CachedGuild, member::CachedMember, role::CachedRole};

fn upsert_guild_item<K: Eq + Hash, V: Eq + Hash>(map: &DashMap<K, HashSet<V>>, k: K, v: V) {
    match map.entry(k) {
        Entry::Occupied(e) if e.get().contains(&v) => {}
        Entry::Occupied(mut e) => {
            e.get_mut().insert(v);
        }
        Entry::Vacant(_) => {
            //PROBLEM TODO: PRINT ERROR
        }
    }
}

fn upsert_item<K: Eq + Hash, V: PartialEq>(map: &DashMap<K, Arc<V>>, k: K, v: V) -> Arc<V> {
    match map.entry(k) {
        Entry::Occupied(e) if **e.get() == v => Arc::clone(e.get()),
        Entry::Occupied(mut e) => {
            let v = Arc::new(v);
            e.insert(Arc::clone(&v));
            v
        }
        Entry::Vacant(e) => {
            let v = Arc::new(v);
            e.insert(Arc::clone(&v));
            v
        }
    }
}

#[derive(Debug, Default)]
pub struct CacheRef {
    channels: DashMap<ChannelId, Arc<GuildChannel>>,
    guilds: DashMap<GuildId, Arc<CachedGuild>>,
    members: DashMap<(GuildId, UserId), Arc<CachedMember>>,
    roles: DashMap<RoleId, Arc<CachedRole>>,
    users: DashMap<UserId, Arc<User>>,

    guild_roles: DashMap<GuildId, HashSet<RoleId>>,
    guild_channels: DashMap<GuildId, HashSet<ChannelId>>,
    guild_members: DashMap<GuildId, HashSet<UserId>>,
    unavailable_guilds: DashSet<GuildId>,

    current_user: Mutex<Option<Arc<CurrentUser>>>,
}

#[derive(Clone, Debug, Default)]
pub struct Cache(Arc<CacheRef>);

#[derive(Debug, Clone)]
pub struct CacheError;

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_user(&self) -> Option<Arc<CurrentUser>> {
        self.0.current_user.lock().expect("Current user poisoned").clone()
    }

    pub fn channel(&self, channel_id: ChannelId) -> Option<Arc<GuildChannel>> {
        self.0.channels.get(&channel_id).map(|c| Arc::clone(c.value()))
    }

    pub fn guild(&self, guild_id: GuildId) -> Option<Arc<CachedGuild>> {
        self.0.guilds.get(&guild_id).map(|g| Arc::clone(g.value()))
    }

    pub fn guilds(&self) -> Vec<u64> {
        self.0.guilds.iter().map(|g| g.id.0).collect::<Vec<_>>()
    }

    pub fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<Arc<CachedMember>> {
        self.0.members.get(&(guild_id, user_id)).map(|m| Arc::clone(m.value()))
    }

    pub fn members(&self, guild_id: GuildId) -> HashSet<UserId> {
        self.0.guild_members.get(&guild_id).map_or_else(HashSet::new, |g| g.value().clone())
    }

    pub fn member_count(&self, guild_id: GuildId) -> usize {
        self.0.guild_members.get(&guild_id).map_or_else(|| 0, |g| g.value().len())
    }

    pub fn role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        self.0.roles.get(&role_id).map(|r| Arc::clone(r.value()))
    }

    pub fn roles(&self, guild_id: GuildId) -> HashSet<RoleId> {
        self.0.guild_roles.get(&guild_id).map_or_else(HashSet::new, |gr| gr.value().clone())
    }

    pub fn guild_roles(&self, guild_id: GuildId) -> Vec<Arc<CachedRole>> {
        let roles = self.roles(guild_id);
        let mut guild_roles = Vec::new();
        for role_id in roles {
            if let Some(role) = self.role(role_id) {
                guild_roles.push(role);
            }
        }
        guild_roles
    }

    pub fn user(&self, user_id: UserId) -> Option<Arc<User>> {
        self.0.users.get(&user_id).map(|u| Arc::clone(u.value()))
    }

    pub fn update<T: UpdateCache>(&self, value: &T) -> Result<(), CacheError> {
        value.update(self)
    }

    pub fn cache_current_user(&self, mut current_user: CurrentUser) {
        let mut user = self.0.current_user.lock().expect("current user poisoned");
        if let Some(mut user) = user.as_mut() {
            if let Some(user) = Arc::get_mut(&mut user) {
                std::mem::swap(user, &mut current_user);
                return;
            }
        }

        *user = Some(Arc::new(current_user));
    }

    pub fn cache_guild_channels(
        &self,
        guild: GuildId,
        channels: impl IntoIterator<Item = GuildChannel>,
    ) -> HashSet<ChannelId> {
        let mut c = HashSet::new();
        for channel in channels.into_iter() {
            let id = channel.id();
            self.cache_guild_channel(guild, channel);
            c.insert(id);
        }
        c
    }

    pub fn cache_guild_channel(&self, guild: GuildId, channel: GuildChannel) -> Arc<GuildChannel> {
        if let GuildChannel::Text(tc) = &channel {
            if tc.name.eq_ignore_ascii_case("rowifi-logs") {
                if let Some(mut guild) = self.0.guilds.get_mut(&guild) {
                    let mut guild = Arc::make_mut(&mut guild);
                    guild.log_channel = Some(channel.id());
                }
            }
        }
        let id = channel.id();
        upsert_guild_item(&self.0.guild_channels, guild, id);
        upsert_item(&self.0.channels, id, channel)
    }

    pub fn cache_members(
        &self,
        guild: GuildId,
        members: impl IntoIterator<Item = Member>,
    ) -> HashSet<UserId> {
        let mut m = HashSet::new();
        for member in members.into_iter() {
            let id = member.user.id;
            self.cache_member(guild, member);
            m.insert(id);
        }
        m
    }

    pub fn cache_member(&self, guild: GuildId, member: Member) -> Arc<CachedMember> {
        let key = (guild, member.user.id);
        match self.0.members.get(&key) {
            Some(m) if **m == member => return Arc::clone(&m),
            _ => {}
        }

        let user = self.cache_user(member.user);
        let cached = Arc::new(CachedMember {
            roles: member.roles,
            nick: member.nick,
            user,
        });
        upsert_guild_item(&self.0.guild_members, guild, cached.user.id);
        self.0.members.insert(key, Arc::clone(&cached));
        cached
    }

    pub fn cache_roles(
        &self,
        guild: GuildId,
        roles: impl IntoIterator<Item = Role>,
    ) -> HashSet<RoleId> {
        let mut r = HashSet::new();
        for role in roles.into_iter() {
            let id = role.id;
            self.cache_role(guild, role);
            r.insert(id);
        }
        r
    }

    pub fn cache_role(&self, guild: GuildId, role: Role) -> Arc<CachedRole> {
        self.cache_extra_roles(guild, role.clone());

        let role = CachedRole {
            id: role.id,
            guild_id: guild,
            name: role.name,
            position: role.position,
            permissions: role.permissions,
        };
        upsert_guild_item(&self.0.guild_roles, guild, role.id);
        upsert_item(&self.0.roles, role.id, role)
    }

    pub fn cache_extra_roles(&self, guild: GuildId, role: Role) {
        if let Some(mut guild) = self.0.guilds.get_mut(&guild) {
            if role.name.eq_ignore_ascii_case("RoWifi Bypass") {
                let mut guild = Arc::make_mut(&mut guild);
                guild.bypass_role = Some(role.id);
            } else if role.name.eq_ignore_ascii_case("RoWifi Nickname Bypass") {
                let mut guild = Arc::make_mut(&mut guild);
                guild.nickname_bypass = Some(role.id);
            } else if role.name.eq_ignore_ascii_case("RoWifi Admin") {
                let mut guild = Arc::make_mut(&mut guild);
                guild.admin_role = Some(role.id);
            }
        }
    }

    pub fn cache_guild(&self, guild: Guild) {
        self.0.guild_roles.insert(guild.id, HashSet::new());
        self.0.guild_channels.insert(guild.id, HashSet::new());
        self.0.guild_members.insert(guild.id, HashSet::new());

        let bypass_role  = guild.roles.iter()
            .find(|(_, r)| r.name.eq_ignore_ascii_case("RoWifi Bypass"))
            .map(|(_, r)| r.id);
        let nickname_bypass = guild.roles.iter()
            .find(|(_, r)| r.name.eq_ignore_ascii_case("RoWifi Nickname Bypass"))
            .map(|(_, r)| r.id);
        let log_channel = guild.channels.iter()
            .find(|(_, c)| c.name().eq_ignore_ascii_case("rowifi-logs"))
            .map(|(_, c)| c.id());
        let admin_role = guild.roles.iter()
            .find(|(_, r)| r.name.eq_ignore_ascii_case("RoWifi Admin"))
            .map(|(_, r)| r.id);

        self.cache_guild_channels(guild.id, guild.channels.into_iter().map(|(_, v)| v));
        self.cache_roles(guild.id, guild.roles.into_iter().map(|(_, r)| r));
        self.cache_members(guild.id, guild.members.into_iter().map(|(_, m)| m));

        let cached = CachedGuild {
            id: guild.id,
            description: guild.description,
            icon: guild.icon,
            joined_at: guild.joined_at,
            member_count: guild.member_count,
            name: guild.name,
            owner_id: guild.owner_id,
            permissions: guild.permissions,
            preferred_locale: guild.preferred_locale,
            unavailable: guild.unavailable,
            log_channel,
            bypass_role,
            nickname_bypass,
            admin_role
        };

        self.0.unavailable_guilds.remove(&guild.id);
        self.0.guilds.insert(guild.id, Arc::new(cached));
    }

    pub fn cache_user(&self, user: User) -> Arc<User> {
        match self.0.users.get(&user.id) {
            Some(u) if **u == user => return Arc::clone(&u),
            _ => {}
        }

        let user = Arc::new(user);
        self.0.users.insert(user.id, Arc::clone(&user));
        user
    }

    pub fn delete_guild_channel(&self, tc: GuildChannel) -> Option<Arc<GuildChannel>> {
        let channel = self.0.channels.remove(&tc.id()).map(|(_, c)| c)?;
        if let Some(mut channels) = self.0.guild_channels.get_mut(&tc.guild_id().unwrap()) {
            channels.remove(&tc.id());
        }
        if channel.name().eq_ignore_ascii_case("rowifi-logs") {
            if let Some(mut guild) = self.0.guilds.get_mut(&tc.guild_id().unwrap()) {
                let mut guild = Arc::make_mut(&mut guild);
                guild.log_channel = None;
            }
        }
        Some(channel)
    }

    pub fn delete_role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        let role = self.0.roles.remove(&role_id).map(|(_, r)| r)?;
        if let Some(mut roles) = self.0.guild_roles.get_mut(&role.guild_id) {
            roles.remove(&role_id);
        }
        if role.name.eq_ignore_ascii_case("RoWifi Bypass") {
            if let Some(mut guild) = self.0.guilds.get_mut(&role.guild_id) {
                let mut guild = Arc::make_mut(&mut guild);
                guild.bypass_role = None;
            }
        } else if role.name.eq_ignore_ascii_case("RoWifi Nickname Bypass") {
            if let Some(mut guild) = self.0.guilds.get_mut(&role.guild_id) {
                let mut guild = Arc::make_mut(&mut guild);
                guild.nickname_bypass = None;
            }
        }
        Some(role)
    }

    pub fn unavailable_guild(&self, guild_id: GuildId) {
        self.0.unavailable_guilds.insert(guild_id);
        self.0.guilds.remove(&guild_id);
    }
}
