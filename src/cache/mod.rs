use dashmap::{mapref::entry::Entry, DashMap, DashSet};
use std::{
    collections::HashSet,
    hash::Hash,
    sync::{Arc, Mutex},
};
use twilight::model::{
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
pub struct Cache {
    channels: DashMap<ChannelId, Arc<GuildChannel>>,
    guilds: DashMap<GuildId, Arc<CachedGuild>>,
    members: DashMap<(GuildId, UserId), Arc<CachedMember>>,
    roles: DashMap<RoleId, Arc<CachedRole>>,
    users: DashMap<UserId, Arc<User>>,

    guild_roles: DashMap<GuildId, HashSet<RoleId>>,
    guild_channels: DashMap<GuildId, HashSet<ChannelId>>,
    guild_members: DashMap<GuildId, HashSet<UserId>>,

    log_channels: DashMap<GuildId, Arc<ChannelId>>,
    bypass_role: DashMap<GuildId, Arc<(Option<RoleId>, Option<RoleId>)>>,
    unavailable_guilds: DashSet<GuildId>,

    current_user: Mutex<Option<Arc<CurrentUser>>>,
}

#[derive(Debug, Clone)]
pub struct CacheError;

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn guild(&self, guild_id: GuildId) -> Option<Arc<CachedGuild>> {
        self.guilds.get(&guild_id).map(|g| Arc::clone(g.value()))
    }

    pub fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<Arc<CachedMember>> {
        self.members
            .get(&(guild_id, user_id))
            .map(|m| Arc::clone(m.value()))
    }

    pub fn bypass_roles(&self, guild_id: GuildId) -> Arc<(Option<RoleId>, Option<RoleId>)> {
        self.bypass_role
            .get(&guild_id)
            .map_or_else(|| Arc::new((None, None)), |b| Arc::clone(b.value()))
    }

    pub fn role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        self.roles.get(&role_id).map(|r| Arc::clone(r.value()))
    }

    pub fn roles(&self, guild_id: GuildId) -> HashSet<RoleId> {
        self.guild_roles
            .get(&guild_id)
            .map_or_else(HashSet::new, |gr| gr.value().clone())
    }

    pub fn update<T: UpdateCache>(&self, value: &T) -> Result<(), CacheError> {
        value.update(self)
    }

    pub fn cache_current_user(&self, mut current_user: CurrentUser) {
        let mut user = self.current_user.lock().expect("current user poisoned");
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
                upsert_item(&self.log_channels, tc.guild_id.unwrap(), tc.id);
            }
        }
        let id = channel.id();
        upsert_guild_item(&self.guild_channels, guild, id);
        upsert_item(&self.channels, id, channel)
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
        match self.members.get(&key) {
            Some(m) if **m == member => return Arc::clone(&m),
            _ => {}
        }

        let user = self.cache_user(member.user);
        let cached = Arc::new(CachedMember {
            roles: member.roles,
            nick: member.nick,
            user,
        });
        upsert_guild_item(&self.guild_members, guild, cached.user.id);
        self.members.insert(key, Arc::clone(&cached));
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
        self.cache_bypass_role(guild, role.clone());

        let role = CachedRole {
            id: role.id,
            guild_id: guild,
            name: role.name,
            position: role.position,
            permissions: role.permissions,
        };
        upsert_guild_item(&self.guild_roles, guild, role.id);
        upsert_item(&self.roles, role.id, role)
    }

    pub fn cache_bypass_role(&self, guild: GuildId, role: Role) {
        if let Some(mut bypass) = self.bypass_role.get_mut(&guild) {
            if role.name.eq_ignore_ascii_case("RoWifi Bypass") {
                let mut bypass = Arc::make_mut(&mut bypass);
                bypass.0 = Some(role.id);
            } else if role.name.eq_ignore_ascii_case("RoWifi Nickname Bypass") {
                let mut bypass = Arc::make_mut(&mut bypass);
                bypass.1 = Some(role.id);
            }
        }
    }

    pub fn cache_guild(&self, guild: Guild) {
        self.guild_roles.insert(guild.id, HashSet::new());
        self.bypass_role.insert(guild.id, Arc::new((None, None)));

        self.cache_guild_channels(guild.id, guild.channels.into_iter().map(|(_, v)| v));
        self.cache_roles(guild.id, guild.roles.into_iter().map(|(_, r)| r));
        self.cache_members(guild.id, guild.members.into_iter().map(|(_, m)| m));

        let cached = CachedGuild {
            id: guild.id,
            description: guild.description,
            discovery_splash: guild.discovery_splash,
            embed_channel_id: guild.embed_channel_id,
            embed_enabled: guild.embed_enabled,
            icon: guild.icon,
            joined_at: guild.joined_at,
            large: guild.large,
            member_count: guild.member_count,
            name: guild.name,
            owner: guild.owner,
            owner_id: guild.owner_id,
            permissions: guild.permissions,
            preferred_locale: guild.preferred_locale,
            splash: guild.splash,
            unavailable: guild.unavailable,
        };

        self.unavailable_guilds.remove(&guild.id);
        self.guilds.insert(guild.id, Arc::new(cached));
    }

    pub fn cache_user(&self, user: User) -> Arc<User> {
        match self.users.get(&user.id) {
            Some(u) if **u == user => return Arc::clone(&u),
            _ => {}
        }

        let user = Arc::new(user);
        self.users.insert(user.id, Arc::clone(&user));
        user
    }

    pub fn delete_guild_channel(&self, tc: GuildChannel) -> Option<Arc<GuildChannel>> {
        let channel = self.channels.remove(&tc.id()).map(|(_, c)| c)?;
        if let Some(mut channels) = self.guild_channels.get_mut(&tc.guild_id().unwrap()) {
            channels.remove(&tc.id());
        }
        if let Some(log_channel) = self.log_channels.get(&tc.guild_id().unwrap()) {
            if log_channel.0 == tc.id().0 {
                self.log_channels.remove(&tc.guild_id().unwrap());
            }
        }
        Some(channel)
    }

    pub fn delete_role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        let role = self.roles.remove(&role_id).map(|(_, r)| r)?;
        if let Some(mut roles) = self.guild_roles.get_mut(&role.guild_id) {
            roles.remove(&role_id);
        }
        if let Some(mut bypass) = self.bypass_role.get_mut(&role.guild_id) {
            if bypass.0 == Some(role_id) {
                let mut bypass = Arc::make_mut(&mut bypass);
                bypass.0 = None;
            } else if bypass.1 == Some(role_id) {
                let mut bypass = Arc::make_mut(&mut bypass);
                bypass.1 = None;
            }
        }
        Some(role)
    }

    pub fn unavailable_guild(&self, guild_id: GuildId) {
        self.unavailable_guilds.insert(guild_id);
        self.guilds.remove(&guild_id);
    }
}
