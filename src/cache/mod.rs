use dashmap::{DashMap, DashSet, mapref::entry::Entry};
use std::{
    sync::Arc,
    hash::Hash,
    collections::HashSet
};
use twilight::model::
    {id::{GuildId, UserId, ChannelId, RoleId},
    user::{User, CurrentUser},
    channel::GuildChannel,
    guild::{Guild, Member, Role}
};
use twilight::cache::{UpdateCache, Cache as CacheTrait};
use tokio::sync::Mutex;

mod event;
mod models;
use models::{guild::CachedGuild, member::CachedMember, role::CachedRole};

async fn upsert_guild_item<K: Eq + Hash, V: Eq + Hash>(map: &DashMap<K, DashSet<V>>, k: K, v: V) {
    match map.entry(k) {
        Entry::Occupied(e) if e.get().contains(&v) => {},
        Entry::Occupied(e) => {
            e.get().insert(v);
        },
        Entry::Vacant(e) => {
            //PROBLEM TODO: PRINT ERROR
        }
    }
}

async fn upsert_item<K: Eq + Hash, V: PartialEq>(map: &DashMap<K, Arc<V>>, k: K, v: V) -> Arc<V> {
    match map.entry(k) {
        Entry::Occupied(e) if **e.get() == v => Arc::clone(e.get()),
        Entry::Occupied(mut e) => {
            let v = Arc::new(v);
            e.insert(Arc::clone(&v));
            v
        },
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

    guild_roles: DashMap<GuildId, DashSet<RoleId>>,
    guild_channels: DashMap<GuildId, DashSet<ChannelId>>,
    guild_members: DashMap<GuildId, DashSet<UserId>>,

    log_channels: DashMap<GuildId, Arc<ChannelId>>,
    bypass_role: DashMap<GuildId, Arc<(Option<RoleId>, Option<RoleId>)>>,
    unavailable_guilds: DashSet<GuildId>,

    current_user: Mutex<Option<Arc<CurrentUser>>>
}

#[derive(Debug, Clone)]
pub struct CacheError;

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn update<T: UpdateCache<Self, CacheError>>(&self, value: &T) -> Result<(), CacheError> {
        value.update(self).await
    }

    pub async fn cache_current_user(&self, mut current_user: CurrentUser) {
        let mut user = self.current_user.lock().await;
        if let Some(mut user) = user.as_mut() {
            if let Some(user) = Arc::get_mut(&mut user) {
                std::mem::swap(user, &mut current_user);    
                return;
            }
        }

        *user = Some(Arc::new(current_user));
    }

    pub async fn cache_guild_channels(&self, guild: GuildId, channels: impl IntoIterator<Item = GuildChannel>) -> HashSet<ChannelId> {
        let mut c = HashSet::new();
        for channel in channels.into_iter() {
            let id = channel.id();
            self.cache_guild_channel(guild, channel).await;
            c.insert(id);
        }
        c
    }

    pub async fn cache_guild_channel(&self, guild: GuildId, channel: GuildChannel) -> Arc<GuildChannel> {
        if let GuildChannel::Text(tc) = &channel {
            if tc.name.eq_ignore_ascii_case("rowifi-logs") {
                upsert_item(&self.log_channels, tc.guild_id.unwrap(), tc.id).await;
            }
        }
        let id = channel.id();
        upsert_guild_item(&self.guild_channels, guild, id).await;
        upsert_item(&self.channels, id, channel).await
    }

    pub async fn cache_members(&self, guild: GuildId, members: impl IntoIterator<Item = Member>) -> HashSet<UserId> {
        let mut m = HashSet::new();
        for member in members.into_iter() {
            let id = member.user.id;
            self.cache_member(guild, member).await;
            m.insert(id);
        }
        m
    }

    pub async fn cache_member(&self, guild: GuildId, member: Member) -> Arc<CachedMember> {
        let key = (guild, member.user.id);
        match self.members.get(&key) {
            Some(m) if **m == member => return Arc::clone(&m),
            _ => {}
        }

        let user = self.cache_user(member.user).await;
        let cached = Arc::new(CachedMember {
            roles: member.roles,
            nick: member.nick,
            user
        });
        upsert_guild_item(&self.guild_members, guild, cached.user.id).await;
        self.members.insert(key, Arc::clone(&cached));
        cached
    }

    pub async fn cache_roles(&self, guild: GuildId, roles: impl IntoIterator<Item = Role>) -> HashSet<RoleId> {
        let mut r = HashSet::new();
        for role in roles.into_iter() {
            let id = role.id;
            self.cache_role(guild, role).await;
            r.insert(id);
        }
        r
    }

    pub async fn cache_role(&self, guild: GuildId, role: Role) -> Arc<CachedRole> {
        self.cache_bypass_role(guild, role.clone()).await;
        
        let role = CachedRole {
            id: role.id,
            guild_id: guild,
            name: role.name,
            position: role.position,
            permissions: role.permissions
        };
        upsert_guild_item(&self.guild_roles, guild, role.id).await;
        upsert_item(&self.roles, role.id, role).await
    }

    pub async fn cache_bypass_role(&self, guild: GuildId, role: Role) {
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

    pub async fn cache_guild(&self, guild: Guild) {
        self.guild_roles.insert(guild.id, DashSet::new());
        self.bypass_role.insert(guild.id, Arc::new((None, None)));
        
        self.cache_guild_channels(guild.id, guild.channels.into_iter().map(|(_, v)| v)).await;
        self.cache_roles(guild.id, guild.roles.into_iter().map(|(_, r)| r)).await;
        self.cache_members(guild.id, guild.members.into_iter().map(|(_, m)| m)).await;
        
        let cached = CachedGuild {
            id: guild.id,
            description: guild.description,
            discovery_splash: guild. discovery_splash,
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
            unavailable: guild.unavailable
        };

        self.unavailable_guilds.remove(&guild.id);
        self.guilds.insert(guild.id, Arc::new(cached));
    }

    pub async fn cache_user(&self, user: User) -> Arc<User> {
        match self.users.get(&user.id) {
            Some(u) if **u == user => return Arc::clone(&u),
            _ => {}
        }

        let user = Arc::new(user);
        self.users.insert(user.id, Arc::clone(&user));
        user
    }

    pub async fn delete_guild_channel(&self, tc: GuildChannel) -> Option<Arc<GuildChannel>> {
        let channel = self.channels.remove(&tc.id()).map(|(_, c)| c)?;
        if let Some(channels) = self.guild_channels.get_mut(&tc.guild_id().unwrap()) {
            channels.remove(&tc.id());
        }
        if let Some(log_channel) = self.log_channels.get(&tc.guild_id().unwrap()) {
            if log_channel.0 == tc.id().0 {
                self.log_channels.remove(&tc.guild_id().unwrap());
            }
        }
        Some(channel)
    }

    pub async fn delete_role(&self, role_id: RoleId) -> Option<Arc<CachedRole>> {
        let role = self.roles.remove(&role_id).map(|(_, r)| r)?;
        if let Some(roles) = self.guild_roles.get_mut(&role.guild_id) {
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

    pub async fn unavailable_guild(&self, guild_id: GuildId) {
        self.unavailable_guilds.insert(guild_id);
        self.guilds.remove(&guild_id);
    }
}

impl CacheTrait for Cache {}
impl CacheTrait for &'_ Cache {}