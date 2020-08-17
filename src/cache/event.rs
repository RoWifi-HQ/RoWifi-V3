use async_trait::async_trait;
use twilight::{
    cache::UpdateCache,
    model::{
        gateway::{payload::*, event::Event},
        channel::Channel,
        guild::GuildStatus
    }
};
use std::{ops::Deref, sync::Arc};
use tracing::debug;

use super::{Cache, CacheError};

#[async_trait]
impl UpdateCache<Cache, CacheError> for Event {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        use Event::*;

        match self {
            ChannelCreate(v) => c.update(v).await,
            ChannelDelete(v) => c.update(v).await,
            ChannelUpdate(v) => c.update(v).await,
            GuildCreate(v) => c.update(v.deref()).await,
            GuildDelete(v) => c.update(v.deref()).await,
            GuildUpdate(v) => c.update(v.deref()).await,
            MemberAdd(v) => c.update(v.deref()).await,
            MemberChunk(v) => c.update(v.deref()).await,
            MemberRemove(v) => c.update(v.deref()).await,
            MemberUpdate(v) => c.update(v.deref()).await,
            Ready(v) => c.update(v.deref()).await,
            RoleCreate(v) => c.update(v.deref()).await,
            RoleDelete(v) => c.update(v.deref()).await,
            RoleUpdate(v) => c.update(v.deref()).await,
            UnavailableGuild(v) => c.update(v).await,
            UserUpdate(v) => c.update(v).await,
            _ => Ok(())
        }
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for ChannelCreate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            let guild_id = gc.guild_id().unwrap();
            c.cache_guild_channel(guild_id, gc).await;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for ChannelDelete {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            c.delete_guild_channel(gc).await;
        }
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for ChannelUpdate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if let Channel::Guild(gc) = self.0.clone() {
            let guild_id = gc.guild_id().unwrap();
            c.cache_guild_channel(guild_id, gc).await;
        }

        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for GuildCreate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        println!("{:?} {:?}", self.0.id, self.0.members.len());
        c.cache_guild(self.0.clone()).await;
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for GuildDelete {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
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

#[async_trait]
impl UpdateCache<Cache, CacheError> for GuildUpdate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        let mut guild = match c.guilds.get_mut(&self.0.id).map(|r| Arc::clone(r.value())) {
            Some(guild) => guild,
            None => return Ok(())
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

#[async_trait]
impl UpdateCache<Cache, CacheError> for MemberAdd {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_member(self.guild_id, self.0.clone()).await;
        Ok(())
    }
}


#[async_trait]
impl UpdateCache<Cache, CacheError> for MemberChunk {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        if self.members.is_empty() {
            return Ok(())
        }

        c.cache_members(self.guild_id, self.members.values().cloned()).await;
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for MemberRemove {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.members.remove(&(self.guild_id, self.user.id));
        if let Some(mut members) = c.guild_members.get_mut(&self.guild_id) {
            members.remove(&self.user.id);
        }
        
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for MemberUpdate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        debug!(id = ?self.user.id, "Received event for Member Update for");
        let mut member = match c.members.get_mut(&(self.guild_id, self.user.id)) {
            Some(member) => member,
            None => return Ok(())
        };
        let mut member = Arc::make_mut(&mut member);

        member.nick = self.nick.clone();
        member.roles = self.roles.clone();

        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for Ready {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.user.clone()).await;

        for status in self.guilds.values() {
            match status {
                GuildStatus::Offline(u) => {
                    c.unavailable_guild(u.id).await
                },
                GuildStatus::Online(g) => {
                    c.cache_guild(g.clone()).await
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for RoleCreate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_role(self.guild_id, self.role.clone()).await;
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for RoleDelete {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.delete_role(self.role_id).await;
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for RoleUpdate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_role(self.guild_id, self.role.clone()).await;
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for UnavailableGuild {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.guilds.remove(&self.id);
        c.unavailable_guilds.insert(self.id);
        Ok(())
    }
}

#[async_trait]
impl UpdateCache<Cache, CacheError> for UserUpdate {
    async fn update(&self, c: &Cache) -> Result<(), CacheError> {
        c.cache_current_user(self.0.clone()).await;
        Ok(())
    }
}