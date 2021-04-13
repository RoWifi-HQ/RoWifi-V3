#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::cast_possible_wrap,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::manual_find_map,
    clippy::implicit_hasher,
    clippy::missing_panics_doc,
    clippy::explicit_deref_methods
)]

use dashmap::{mapref::entry::Entry, DashMap, DashSet};
use rowifi_models::stats::BotStats;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex,
    },
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwriteType, GuildChannel},
    guild::{Guild, Member, Permissions, Role},
    id::{ChannelId, GuildId, RoleId, UserId},
    user::{CurrentUser, User},
};

mod event;
mod models;
use event::UpdateCache;
pub use models::{guild::CachedGuild, member::CachedMember, role::CachedRole};

/// Add an element to the structure that maps the server ids to the set of the resource they hold
fn upsert_guild_item<K: Eq + Hash, V: Eq + Hash>(map: &DashMap<K, HashSet<V>>, k: K, v: V) {
    match map.entry(k) {
        Entry::Occupied(e) if e.get().contains(&v) => {}
        Entry::Occupied(mut e) => {
            e.get_mut().insert(v);
        }
        Entry::Vacant(_) => {}
    }
}

/// Add or modify an element that maps the resource ids to their respective structures
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

    guild_permissions: DashMap<GuildId, Permissions>,
    channel_permissions: DashMap<ChannelId, Permissions>,

    stats: Arc<BotStats>,
}

/// An wrapper around the actual structure that hold all the cache field allowing this to be sent across multiple threads
#[derive(Clone)]
pub struct Cache(Arc<CacheRef>);

#[derive(Debug, Clone)]
pub struct CacheError;

impl Cache {
    #[must_use]
    pub fn new(stats: Arc<BotStats>) -> Self {
        Self {
            0: Arc::new(CacheRef {
                channels: DashMap::new(),
                guilds: DashMap::new(),
                members: DashMap::new(),
                roles: DashMap::new(),
                users: DashMap::new(),
                guild_roles: DashMap::new(),
                guild_channels: DashMap::new(),
                guild_members: DashMap::new(),
                unavailable_guilds: DashSet::new(),
                current_user: Mutex::new(None),
                guild_permissions: DashMap::new(),
                channel_permissions: DashMap::new(),
                stats,
            }),
        }
    }

    /// Returns the bot user
    pub fn current_user(&self) -> Option<Arc<CurrentUser>> {
        self.0
            .current_user
            .lock()
            .expect("Current user poisoned")
            .clone()
    }

    /// Get a immutable reference to a channel
    pub fn channel(&self, channel_id: ChannelId) -> Option<Arc<GuildChannel>> {
        self.0
            .channels
            .get(&channel_id)
            .map(|c| Arc::clone(c.value()))
    }

    /// Get a cloned list of the channel ids of a particular guild
    pub fn guild_channels(&self, guild_id: GuildId) -> HashSet<ChannelId> {
        self.0
            .guild_channels
            .get(&guild_id)
            .map_or_else(HashSet::new, |gc| gc.value().clone())
    }

    /// Get the permissions of the bot in a certain channel
    pub fn channel_permissions(&self, channel_id: ChannelId) -> Option<Permissions> {
        self.0
            .channel_permissions
            .get(&channel_id)
            .map(|c| *c.value())
    }

    /// Get an immutable reference to the guild struct
    pub fn guild(&self, guild_id: GuildId) -> Option<Arc<CachedGuild>> {
        self.0.guilds.get(&guild_id).map(|g| Arc::clone(g.value()))
    }

    /// Get a list of all guild ids inside the cache
    pub fn guilds(&self) -> Vec<u64> {
        self.0.guilds.iter().map(|g| g.id.0).collect::<Vec<_>>()
    }

    /// Get an immutable reference to a certain user in a certain guild
    pub fn member(&self, guild_id: GuildId, user_id: UserId) -> Option<Arc<CachedMember>> {
        self.0
            .members
            .get(&(guild_id, user_id))
            .map(|m| Arc::clone(m.value()))
    }

    /// Get a list of all member ids inside a guild
    pub fn members(&self, guild_id: GuildId) -> HashSet<UserId> {
        self.0
            .guild_members
            .get(&guild_id)
            .map_or_else(HashSet::new, |g| g.value().clone())
    }

    /// Get the membercount of a guild. Returns 0 if the guild is not present inside the cache
    pub fn member_count(&self, guild_id: GuildId) -> i64 {
        self.0
            .guilds
            .get(&guild_id)
            .map_or_else(|| 0, |g| g.member_count.load(Ordering::SeqCst))
    }

    /// Get an immutable reference of a certain role
    pub fn role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        self.0.roles.get(&role_id).map(|r| Arc::clone(r.value()))
    }

    /// Get a list of all role ids inside a guild
    pub fn roles(&self, guild_id: GuildId) -> HashSet<RoleId> {
        self.0
            .guild_roles
            .get(&guild_id)
            .map_or_else(HashSet::new, |gr| gr.value().clone())
    }

    /// Get a list of all role structs inside a guild
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

    /// Get an immutable reference to a certain user
    pub fn user(&self, user_id: UserId) -> Option<Arc<User>> {
        self.0.users.get(&user_id).map(|u| Arc::clone(u.value()))
    }

    /// Update a resource inside a cache
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
        for channel in channels {
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
        for member in members {
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
            pending: member.pending,
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
        for role in roles {
            let id = role.id;
            self.cache_role(guild, role);
            r.insert(id);
        }
        r
    }

    pub fn cache_role(&self, guild: GuildId, role: Role) -> Arc<CachedRole> {
        self.cache_extra_roles(guild, &role);

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

    pub fn cache_extra_roles(&self, guild: GuildId, role: &Role) {
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
            } else if role.name.eq_ignore_ascii_case("RoWifi Trainer") {
                let mut guild = Arc::make_mut(&mut guild);
                guild.trainer_role = Some(role.id);
            }
        }
    }

    pub fn cache_guild_permissions(&self, guild_id: GuildId) {
        let user = self.0.current_user.lock().expect("current user poisoned");
        if let Some(user) = user.as_ref() {
            let guild = self.guild(guild_id).unwrap();
            let server_roles = self
                .guild_roles(guild_id)
                .iter()
                .map(|r| (r.id, r.clone()))
                .collect::<HashMap<RoleId, Arc<CachedRole>>>();
            let member = self.member(guild_id, user.id).unwrap();
            let new_permissions =
                match guild_wide_permissions(&guild, &server_roles, user.id, &member.roles) {
                    Ok(p) => p,
                    Err(why) => {
                        tracing::error!(guild = ?guild_id, reason = ?why);
                        return;
                    }
                };
            self.0.guild_permissions.insert(guild_id, new_permissions);
        }
    }

    pub fn cache_channel_permissions(&self, guild_id: GuildId, channel_id: ChannelId) {
        let channel = self.channel(channel_id).unwrap();
        if let GuildChannel::Text(_) = channel.as_ref() {
            let user = self.0.current_user.lock().expect("current user poisoned");
            if let Some(user) = user.as_ref() {
                let guild = self.guild(guild_id).unwrap();
                let server_roles = self
                    .guild_roles(guild_id)
                    .iter()
                    .map(|r| (r.id, r.clone()))
                    .collect::<HashMap<RoleId, Arc<CachedRole>>>();
                let member = self.member(guild_id, user.id).unwrap();
                let new_permissions = match channel_permissions(
                    &guild,
                    &server_roles,
                    user.id,
                    &member.roles,
                    &channel,
                ) {
                    Ok(p) => p,
                    Err(why) => {
                        tracing::error!(guild = ?guild_id, channel = ?channel_id, reason = ?why);
                        return;
                    }
                };
                self.0
                    .channel_permissions
                    .insert(channel_id, new_permissions);
            }
        }
    }

    pub fn cache_guild(&self, guild: Guild) -> Option<Arc<CachedGuild>> {
        self.0.guild_roles.insert(guild.id, HashSet::new());
        self.0.guild_channels.insert(guild.id, HashSet::new());
        if !self.0.guild_members.contains_key(&guild.id) {
            self.0.guild_members.insert(guild.id, HashSet::new());
        }

        let bypass_role = guild
            .roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case("RoWifi Bypass"))
            .map(|r| r.id);
        let nickname_bypass = guild
            .roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case("RoWifi Nickname Bypass"))
            .map(|r| r.id);
        let log_channel = guild
            .channels
            .iter()
            .find(|c| c.name().eq_ignore_ascii_case("rowifi-logs"))
            .map(GuildChannel::id);
        let admin_role = guild
            .roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case("RoWifi Admin"))
            .map(|r| r.id);
        let trainer_role = guild
            .roles
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case("RoWifi Trainer"))
            .map(|r| r.id);

        self.cache_guild_channels(guild.id, guild.channels.into_iter());
        self.cache_roles(guild.id, guild.roles.into_iter());
        self.cache_members(guild.id, guild.members.into_iter());

        let cached = CachedGuild {
            id: guild.id,
            description: guild.description,
            icon: guild.icon,
            joined_at: guild.joined_at,
            member_count: Arc::new(AtomicI64::new(0)),
            name: guild.name,
            owner_id: guild.owner_id,
            permissions: guild.permissions,
            preferred_locale: guild.preferred_locale,
            unavailable: guild.unavailable,
            log_channel,
            bypass_role,
            nickname_bypass,
            admin_role,
            trainer_role,
        };

        self.0.unavailable_guilds.remove(&guild.id);
        self.0.guilds.insert(guild.id, Arc::new(cached))
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

    pub fn delete_guild_channel(&self, tc: &GuildChannel) -> Option<Arc<GuildChannel>> {
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
        } else if role.name.eq_ignore_ascii_case("RoWifi Admin") {
            if let Some(mut guild) = self.0.guilds.get_mut(&role.guild_id) {
                let mut guild = Arc::make_mut(&mut guild);
                guild.admin_role = None;
            }
        } else if role.name.eq_ignore_ascii_case("RoWifi Trainer") {
            if let Some(mut guild) = self.0.guilds.get_mut(&role.guild_id) {
                let mut guild = Arc::make_mut(&mut guild);
                guild.trainer_role = None;
            }
        }
        Some(role)
    }

    pub fn unavailable_guild(&self, guild_id: GuildId) {
        self.0.unavailable_guilds.insert(guild_id);
    }
}

pub fn guild_wide_permissions(
    guild: &Arc<CachedGuild>,
    roles: &HashMap<RoleId, Arc<CachedRole>>,
    member_id: UserId,
    member_roles: &[RoleId],
) -> Result<Permissions, String> {
    if member_id == guild.owner_id {
        return Ok(Permissions::all());
    }

    let mut permissions = match roles.get(&RoleId(guild.id.0)) {
        Some(r) => r.permissions,
        None => return Err("`@everyone` role is missing from the cache.".into()),
    };

    for role in member_roles {
        let role_permissions = match roles.get(&role) {
            Some(r) => r.permissions,
            None => return Err("Found a role on the member that doesn't exist on the cache".into()),
        };

        permissions |= role_permissions;
    }
    Ok(permissions)
}

pub fn channel_permissions(
    guild: &Arc<CachedGuild>,
    roles: &HashMap<RoleId, Arc<CachedRole>>,
    member_id: UserId,
    member_roles: &[RoleId],
    channel: &Arc<GuildChannel>,
) -> Result<Permissions, String> {
    let guild_id = guild.id;
    let mut permissions = guild_wide_permissions(&guild, roles, member_id, &member_roles)?;
    let mut member_allow = Permissions::empty();
    let mut member_deny = Permissions::empty();
    let mut roles_allow = Permissions::empty();
    let mut roles_deny = Permissions::empty();

    if let GuildChannel::Text(tc) = channel.as_ref() {
        for overwrite in &tc.permission_overwrites {
            match overwrite.kind {
                PermissionOverwriteType::Role(role) => {
                    if role.0 == guild_id.0 {
                        permissions.remove(overwrite.deny);
                        permissions.insert(overwrite.allow);
                        continue;
                    }

                    if !member_roles.contains(&role) {
                        continue;
                    }

                    roles_allow.insert(overwrite.allow);
                    roles_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(user) if user == member_id => {
                    member_allow.insert(overwrite.allow);
                    member_deny.insert(overwrite.deny);
                }
                PermissionOverwriteType::Member(_) => {}
            }
        }
        permissions.remove(roles_deny);
        permissions.insert(roles_allow);
        permissions.remove(member_deny);
        permissions.insert(member_allow);

        return Ok(permissions);
    }

    Err("Not implemented for non text guild channels".into())
}
