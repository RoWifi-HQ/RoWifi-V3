use rowifi_models::{discord::{
    application::interaction::Interaction,
    channel::Channel,
    gateway::{
        event::Event,
        payload::incoming::{
            ChannelCreate, ChannelDelete, ChannelUpdate, GuildCreate, GuildDelete, GuildUpdate,
            InteractionCreate, MemberAdd, MemberChunk, MemberRemove, MemberUpdate, MessageCreate,
            Ready, RoleCreate, RoleDelete, RoleUpdate, UnavailableGuild, UserUpdate,
        },
    },
}, id::GuildId};
use std::{
    ops::Deref,
    sync::{atomic::Ordering, Arc},
};
use tracing::{debug, info};

use super::{Cache, CacheError, CachedMember};

pub trait UpdateCache {
    fn update(&self, cache: &Cache) -> Result<(), CacheError>;
}

impl UpdateCache for Event {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        use Event::{
            ChannelCreate, ChannelDelete, ChannelUpdate, GuildCreate, GuildDelete, GuildUpdate,
            InteractionCreate, MemberAdd, MemberChunk, MemberRemove, MemberUpdate, MessageCreate,
            Ready, RoleCreate, RoleDelete, RoleUpdate, UnavailableGuild, UserUpdate,
        };

        match self {
            ChannelCreate(v) => c.update(v),
            ChannelDelete(v) => c.update(v),
            ChannelUpdate(v) => c.update(v),
            GuildCreate(v) => c.update(v.deref()),
            GuildDelete(v) => c.update(v.deref()),
            GuildUpdate(v) => c.update(v.deref()),
            InteractionCreate(v) => c.update(v.deref()),
            MemberAdd(v) => c.update(v.deref()),
            MemberChunk(v) => c.update(v.deref()),
            MemberRemove(v) => c.update(v.deref()),
            MemberUpdate(v) => c.update(v.deref()),
            MessageCreate(v) => c.update(v.deref()),
            Ready(v) => c.update(v.deref()),
            RoleCreate(v) => c.update(v.deref()),
            RoleDelete(v) => c.update(v.deref()),
            RoleUpdate(v) => c.update(v.deref()),
            UnavailableGuild(v) => c.update(v),
            UserUpdate(v) => c.update(v),
            _ => Ok(()),
        }
    }
}

impl UpdateCache for ChannelCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            let guild_id = gc.guild_id().unwrap();
            c.cache_guild_channel(GuildId(guild_id), gc);
            c.cache_channel_permissions(GuildId(guild_id), self.id());
        }

        Ok(())
    }
}

impl UpdateCache for ChannelDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            c.delete_guild_channel(&gc);
            c.0.channel_permissions.remove(&self.id());
        }
        Ok(())
    }
}

impl UpdateCache for ChannelUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            let guild_id = gc.guild_id().unwrap();
            c.cache_guild_channel(GuildId(guild_id), gc);
            c.cache_channel_permissions(GuildId(guild_id), self.id());
        }

        Ok(())
    }
}

impl UpdateCache for GuildCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let old_guild = c.cache_guild(self.0.clone());
        let guild_id = GuildId(self.id);
        let guild = c.guild(guild_id).unwrap();
        guild
            .member_count
            .store(self.member_count.unwrap() as i64, Ordering::SeqCst);
        c.cache_guild_permissions(guild_id);
        for channel in &self.channels {
            c.cache_channel_permissions(guild_id, channel.id());
        }
        if old_guild.is_none() {
            c.0.stats.resource_counts.guilds.inc();
            c.0.stats
                .resource_counts
                .users
                .add(self.member_count.unwrap() as i64);
        }
        Ok(())
    }
}

impl UpdateCache for GuildDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let guild_id = GuildId(self.id);
        let guild = c.0.guilds.remove(&guild_id);
        c.0.guild_permissions.remove(&guild_id);
        if let Some((_, ids)) = c.0.guild_channels.remove(&guild_id) {
            for id in ids {
                c.0.channels.remove(&id);
                c.0.channel_permissions.remove(&id);
            }
        }
        if let Some((_, ids)) = c.0.guild_roles.remove(&guild_id) {
            for id in ids {
                c.0.roles.remove(&id);
            }
        }
        if let Some((_, ids)) = c.0.guild_members.remove(&guild_id) {
            for id in ids {
                c.0.members.remove(&(guild_id, id));
            }
        }

        if let Some(guild) = guild {
            c.0.stats.resource_counts.guilds.dec();
            c.0.stats
                .resource_counts
                .users
                .sub(guild.1.member_count.load(Ordering::SeqCst));
        }

        Ok(())
    }
}

impl UpdateCache for GuildUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        debug!(id = ?self.id, "Received event for Guild Update for");

        if let Some(mut guild) = c.0.guilds.get_mut(&GuildId(self.0.id)) {
            let mut guild = Arc::make_mut(&mut guild);
            guild.description = self.description.clone();
            guild.icon = self.icon.clone();
            guild.name = self.name.clone();
            guild.owner_id = self.owner_id;
            guild.permissions = self.permissions;
            guild.preferred_locale = self.preferred_locale.clone();
        }

        Ok(())
    }
}

impl UpdateCache for InteractionCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Interaction::ApplicationCommand(inner) = &self.0 {
            if let Some(member) = &inner.member {
                if let Some(user) = &member.user {
                    if let Some(guild_id) = inner.guild_id {
                        let guild_id = GuildId(guild_id);
                        let user = c.cache_user(user.clone());
                        let id = (guild_id, user.id);
                        match c.0.members.get(&id) {
                            Some(m) if **m == member => return Ok(()),
                            _ => {}
                        }

                        c.0.guild_members
                            .entry(guild_id)
                            .or_default()
                            .insert(user.id);

                        let cached = Arc::new(CachedMember {
                            nick: member.nick.clone(),
                            pending: false,
                            roles: member.roles.clone(),
                            user,
                        });
                        c.0.members.insert(id, Arc::clone(&cached));
                    }
                }
            }
        }
        Ok(())
    }
}

impl UpdateCache for MemberAdd {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let guild_id = GuildId(self.guild_id);
        c.cache_member(self.guild_id, self.0.clone());
        if let Some(guild) = c.guild(self.guild_id) {
            guild.member_count.fetch_add(1, Ordering::SeqCst);
            c.0.stats.resource_counts.users.inc();
        }
        Ok(())
    }
}

impl UpdateCache for MemberChunk {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if self.members.is_empty() {
            return Ok(());
        }
        info!(id = ?self.guild_id, "Received event for Guild Members Chunk for");
        c.cache_members(GuildId(self.guild_id), self.members.clone());
        Ok(())
    }
}

impl UpdateCache for MemberRemove {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {            
        let guild_id = GuildId(self.guild_id);
        c.0.members.remove(&(guild_id, self.user.id));
        if let Some(mut members) = c.0.guild_members.get_mut(&guild_id) {
            if let Some(guild) = c.guild(self.guild_id) {
                guild.member_count.fetch_sub(1, Ordering::SeqCst);
                members.remove(&self.user.id);
            }
        }
        c.0.stats.resource_counts.users.dec();
        Ok(())
    }
}

impl UpdateCache for MemberUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        debug!(id = ?self.user.id, "Received event for Member Update for");
        let guild_id = GuildId(self.guild_id);
        {
            if let Some(mut member) = c.0.members.get_mut(&(guild_id, self.user.id)) {
                let mut member = Arc::make_mut(&mut member);

                member.nick = self.nick.clone();
                member.roles = self.roles.clone();
            }
        }

        if let Some(current_user) = c.current_user() {
            if self.user.id == current_user.id {
                c.cache_guild_permissions(guild_id);
                let channels = c.guild_channels(guild_id);
                for channel in channels {
                    c.cache_channel_permissions(guild_id, channel);
                }
            }
        }

        Ok(())
    }
}

impl UpdateCache for MessageCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let user = c.cache_user(self.author.clone());

        if let (Some(member), Some(guild_id)) = (&self.member, self.guild_id) {
            let guild_id = GuildId(guild_id);
            let id = (guild_id, user.id);
            match c.0.members.get(&id) {
                Some(m) if **m == member => return Ok(()),
                _ => {}
            }

            c.0.guild_members
                .entry(guild_id)
                .or_default()
                .insert(user.id);

            let cached = Arc::new(CachedMember {
                nick: member.nick.clone(),
                pending: false,
                roles: member.roles.clone(),
                user,
            });
            c.0.members.insert(id, Arc::clone(&cached));
        }

        Ok(())
    }
}

impl UpdateCache for Ready {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.user.clone());

        for ug in &self.guilds {
            c.unavailable_guild(GuildId(ug.id));
        }

        Ok(())
    }
}

impl UpdateCache for RoleCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_role(GuildId(self.guild_id), self.role.clone());
        Ok(())
    }
}

impl UpdateCache for RoleDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let guild_id = GuildId(self.guild_id);
        c.delete_role(self.role_id);
        c.cache_guild_permissions(guild_id);
        let channels = c.guild_channels(guild_id);
        for channel in channels {
            c.cache_channel_permissions(guild_id, channel);
        }
        Ok(())
    }
}

impl UpdateCache for RoleUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let guild_id = GuildId(self.guild_id);
        c.cache_role(guild_id, self.role.clone());
        c.cache_guild_permissions(guild_id);
        let channels = c.guild_channels(guild_id);
        for channel in channels {
            c.cache_channel_permissions(guild_id, channel);
        }
        Ok(())
    }
}

impl UpdateCache for UnavailableGuild {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let id = GuildId(self.id);
        c.0.guilds.remove(&id);
        c.0.unavailable_guilds.insert(id);
        Ok(())
    }
}

impl UpdateCache for UserUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.0.clone());
        Ok(())
    }
}
