use std::{ops::Deref, sync::Arc};
use tracing::debug;
use twilight::model::{
    channel::Channel,
    gateway::{event::Event, payload::*},
    guild::GuildStatus,
};

use super::{Cache, CacheError};

pub trait UpdateCache {
    fn update(&self, cache: &Cache) -> Result<(), CacheError>;
}

impl UpdateCache for Event {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        use Event::*;

        match self {
            ChannelCreate(v) => c.update(v),
            ChannelDelete(v) => c.update(v),
            ChannelUpdate(v) => c.update(v),
            GuildCreate(v) => c.update(v.deref()),
            GuildDelete(v) => c.update(v.deref()),
            GuildUpdate(v) => c.update(v.deref()),
            MemberAdd(v) => c.update(v.deref()),
            MemberChunk(v) => c.update(v.deref()),
            MemberRemove(v) => c.update(v.deref()),
            MemberUpdate(v) => c.update(v.deref()),
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
            c.cache_guild_channel(guild_id, gc);
        }

        Ok(())
    }
}

impl UpdateCache for ChannelDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            c.delete_guild_channel(gc);
        }
        Ok(())
    }
}

impl UpdateCache for ChannelUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            let guild_id = gc.guild_id().unwrap();
            c.cache_guild_channel(guild_id, gc);
        }

        Ok(())
    }
}

impl UpdateCache for GuildCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        println!("{:?}", self.0.members);
        c.cache_guild(self.0.clone());
        Ok(())
    }
}

impl UpdateCache for GuildDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.guilds.remove(&self.id);

        {
            if let Some((_, ids)) = c.guild_channels.remove(&self.id) {
                for id in ids {
                    c.channels.remove(&id);
                }
            }
            c.log_channels.remove(&self.id);
        }

        {
            if let Some((_, ids)) = c.guild_roles.remove(&self.id) {
                for id in ids {
                    c.roles.remove(&id);
                }
            }
            c.bypass_role.remove(&self.id);
        }

        {
            if let Some((_, ids)) = c.guild_members.remove(&self.id) {
                for id in ids {
                    c.members.remove(&(self.id, id));
                }
            }
        }

        Ok(())
    }
}

impl UpdateCache for GuildUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let mut guild = match c.guilds.get_mut(&self.0.id).map(|r| Arc::clone(r.value())) {
            Some(guild) => guild,
            None => return Ok(()),
        };

        let g = &self.0;
        let mut guild = Arc::make_mut(&mut guild);
        guild.description = g.description.clone();
        guild.embed_channel_id = g.embed_channel_id;
        guild.embed_enabled.replace(g.embed_enabled);
        guild.icon = g.icon.clone();
        guild.name = g.name.clone();
        guild.owner = g.owner;
        guild.owner_id = g.owner_id;
        guild.permissions = g.permissions;
        guild.preferred_locale = g.preferred_locale.clone();
        guild.splash = g.splash.clone();

        Ok(())
    }
}

impl UpdateCache for MemberAdd {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_member(self.guild_id, self.0.clone());
        Ok(())
    }
}

impl UpdateCache for MemberChunk {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if self.members.is_empty() {
            return Ok(());
        }

        c.cache_members(self.guild_id, self.members.values().cloned());
        Ok(())
    }
}

impl UpdateCache for MemberRemove {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.members.remove(&(self.guild_id, self.user.id));
        if let Some(mut members) = c.guild_members.get_mut(&self.guild_id) {
            members.remove(&self.user.id);
        }

        Ok(())
    }
}

impl UpdateCache for MemberUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        debug!(id = ?self.user.id, "Received event for Member Update for");
        let mut member = match c.members.get_mut(&(self.guild_id, self.user.id)) {
            Some(member) => member,
            None => return Ok(()),
        };
        let mut member = Arc::make_mut(&mut member);

        member.nick = self.nick.clone();
        member.roles = self.roles.clone();

        Ok(())
    }
}

impl UpdateCache for Ready {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.user.clone());

        for status in self.guilds.values() {
            match status {
                GuildStatus::Offline(u) => c.unavailable_guild(u.id),
                GuildStatus::Online(g) => c.cache_guild(g.clone()),
            }
        }

        Ok(())
    }
}

impl UpdateCache for RoleCreate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_role(self.guild_id, self.role.clone());
        Ok(())
    }
}

impl UpdateCache for RoleDelete {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.delete_role(self.role_id);
        Ok(())
    }
}

impl UpdateCache for RoleUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_role(self.guild_id, self.role.clone());
        Ok(())
    }
}

impl UpdateCache for UnavailableGuild {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.guilds.remove(&self.id);
        c.unavailable_guilds.insert(self.id);
        Ok(())
    }
}

impl UpdateCache for UserUpdate {
    fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.0.clone());
        Ok(())
    }
}
