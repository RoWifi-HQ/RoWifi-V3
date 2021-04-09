#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::field_reassign_with_default,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::cast_possible_wrap,
    clippy::single_match_else,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc,
    clippy::let_unit_value
)]

pub mod error;

use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, document::Document, Bson},
    options::{
        ClientOptions, FindOneAndDeleteOptions, FindOneAndReplaceOptions, FindOneAndUpdateOptions,
        FindOneOptions, FindOptions, InsertOneOptions, ReturnDocument,
    },
    Client,
};
use rowifi_models::{
    analytics::Group,
    events::EventLog,
    guild::{BackupGuild, GuildType, RoGuild},
    user::{PremiumUser, QueueUser, RoGuildUser, RoUser},
};
use rowifi_redis::{redis::AsyncCommands, RedisPool};
use std::{collections::HashMap, result::Result as StdResult, sync::Arc, time::Duration};
use transient_dashmap::TransientDashMap;

pub use error::DatabaseError;

type Result<T> = StdResult<T, DatabaseError>;

#[derive(Clone)]
pub struct Database {
    client: Client,
    guild_cache: Arc<TransientDashMap<i64, Arc<RoGuild>>>,
    user_cache: Arc<TransientDashMap<i64, Arc<RoUser>>>,
    redis_pool: RedisPool,
}

impl Database {
    pub async fn new(conn_string: &str, redis_pool: RedisPool) -> Self {
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        Self {
            client,
            guild_cache: Arc::new(TransientDashMap::new(Duration::from_secs(6 * 3600))),
            user_cache: Arc::new(TransientDashMap::new(Duration::from_secs(6 * 3600))),
            redis_pool,
        }
    }

    pub async fn add_guild(&self, guild: RoGuild, replace: bool) -> Result<()> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let guild_bson = bson::to_bson(&guild)?;
        let key = format!("database:g:{}", guild.id);
        let mut conn = self.redis_pool.get().await?;
        if let Bson::Document(g) = guild_bson {
            if replace {
                let mut options = FindOneAndReplaceOptions::default();
                options.return_document = Some(ReturnDocument::After);
                let res = guilds
                    .find_one_and_replace(doc! {"_id": guild.id}, g, options)
                    .await?;
                if let Some(res) = res {
                    let guild = bson::from_bson::<RoGuild>(Bson::Document(res))?;
                    let _: () = conn.set_ex(key, guild, 6 * 3600).await.unwrap();
                }
            } else {
                let _res = guilds.insert_one(g, InsertOneOptions::default()).await?;
                let _: () = conn.set_ex(key, guild, 6 * 3600).await.unwrap();
            }
        }
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: u64) -> Result<Option<Arc<RoGuild>>> {
        let guild_id = guild_id as i64;
        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:g:{}", guild_id);
        let guild: Option<RoGuild> = conn.get(&key).await?;
        match guild {
            Some(g) => Ok(Some(Arc::new(g))),
            None => {
                let guilds = self.client.database("RoWifi").collection("guilds");
                let result = guilds
                    .find_one(doc! {"_id": guild_id}, FindOneOptions::default())
                    .await?;
                let guild = match result {
                    None => return Ok(None),
                    Some(res) => bson::from_bson::<RoGuild>(Bson::Document(res))?,
                };
                let _: () = conn.set_ex(key, guild.clone(), 6 * 3600).await.unwrap();
                Ok(Some(Arc::new(guild)))
            }
        }
    }

    pub async fn get_guilds(&self, guild_ids: &[u64], premium_only: bool) -> Result<Vec<RoGuild>> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let filter = if premium_only {
            doc! {"Settings.AutoDetection": true, "_id": {"$in": guild_ids}}
        } else {
            doc! {"_id": {"$in": guild_ids}}
        };
        let mut cursor = guilds.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(ref document) => {
                    let doc = Bson::Document(document.clone());
                    match bson::from_bson::<RoGuild>(doc) {
                        Ok(guild) => result.push(guild),
                        Err(e) => {
                            tracing::error!(error = ?e, doc = ?document, "Error in deserializing")
                        }
                    }
                }
                Err(e) => tracing::error!(error = ?e, "Error in the cursor"),
            }
        }
        Ok(result)
    }

    pub async fn modify_guild(&self, filter: Document, update: Document) -> Result<()> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        let res = guilds
            .find_one_and_update(filter, update, options)
            .await?
            .unwrap();
        let guild = bson::from_bson::<RoGuild>(Bson::Document(res))?;

        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:g:{}", guild.id);
        let _: () = conn.set_ex(key, guild, 6 * 3600).await?;
        Ok(())
    }

    pub async fn add_queue_user(&self, user: QueueUser) -> Result<()> {
        let queue = self.client.database("RoWifi").collection("queue");

        let exists = queue
            .find_one(doc! {"_id": user.roblox_id}, FindOneOptions::default())
            .await?
            .is_some();

        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:u:{}", user.discord_id);

        let user_doc = bson::to_bson(&user)?;
        if let Bson::Document(u) = user_doc {
            if exists {
                let _res = queue
                    .find_one_and_replace(
                        doc! {"_id": user.roblox_id},
                        u,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _res = queue.insert_one(u, InsertOneOptions::default()).await?;
            }
        }
        conn.del(key).await?;
        Ok(())
    }

    pub async fn add_user(&self, user: RoUser, verified: bool) -> Result<()> {
        let users = self.client.database("RoWifi").collection("users");
        let user_doc = bson::to_bson(&user)?;
        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:u:{}", user.discord_id);
        if let Bson::Document(u) = user_doc {
            if verified {
                let _res = users
                    .find_one_and_replace(
                        doc! {"_id": user.discord_id},
                        u,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _res = users.insert_one(u, InsertOneOptions::default()).await?;
            }
            let _: () = conn.set_ex(key, user, 6 * 3600).await?;
        }
        Ok(())
    }

    pub async fn add_linked_user(&self, linked_user: RoGuildUser) -> Result<()> {
        let linked_users = self.client.database("RoWifi").collection("linked_users");
        let old_linked_user = self
            .get_linked_user(linked_user.discord_id as u64, linked_user.guild_id as u64)
            .await?;
        let linked_user_doc = bson::to_document(&linked_user)?;
        if let Some(old_linked_user) = old_linked_user {
            let _res = linked_users
                .find_one_and_replace(doc! {"GuildId": old_linked_user.guild_id, "UserId": old_linked_user.discord_id}, linked_user_doc, None)
                .await?;
        } else {
            let _res = linked_users.insert_one(linked_user_doc, None).await?;
        }
        Ok(())
    }

    pub async fn delete_linked_users(&self, user_id: u64, roblox_id: i64) -> Result<()> {
        let linked_users = self.client.database("RoWifi").collection("linked_users");
        let filter = doc! {"UserId": user_id, "RobloxId": roblox_id};
        let _res = linked_users.delete_many(filter, None).await?;
        Ok(())
    }

    pub async fn get_user(&self, user_id: u64) -> Result<Option<Arc<RoUser>>> {
        let user_id = user_id as i64;

        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:u:{}", user_id);

        let user: Option<RoUser> = conn.get(&key).await?;
        match user {
            Some(u) => Ok(Some(Arc::new(u))),
            None => {
                let users = self.client.database("RoWifi").collection("users");
                let result = users
                    .find_one(doc! {"_id": user_id}, FindOneOptions::default())
                    .await?;
                let user = match result {
                    None => return Ok(None),
                    Some(res) => bson::from_bson::<RoUser>(Bson::Document(res))?,
                };
                let _: () = conn.set_ex(key, user.clone(), 6 * 3600).await?;
                Ok(Some(Arc::new(user)))
            }
        }
    }

    pub async fn get_linked_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Option<RoGuildUser>> {
        let linked_users = self.client.database("RoWifi").collection("linked_users");
        let result = linked_users
            .find_one(doc! {"GuildId": guild_id, "UserId": user_id}, None)
            .await?;
        match result {
            None => Ok(None),
            Some(doc) => Ok(Some(bson::from_document::<RoGuildUser>(doc)?)),
        }
    }

    pub async fn get_users(&self, user_ids: &[u64]) -> Result<Vec<RoUser>> {
        let users = self.client.database("RoWifi").collection("users");
        let filter = doc! {"_id": {"$in": user_ids}};
        let mut cursor = users.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<RoUser>(Bson::Document(document))?),
                Err(e) => tracing::error!(err = ?e),
            }
        }
        Ok(result)
    }

    pub async fn get_linked_users(
        &self,
        user_ids: &[u64],
        guild_id: u64,
    ) -> Result<Vec<RoGuildUser>> {
        let linked_users = self.client.database("RoWifi").collection("linked_users");
        let filter = doc! {"UserId": {"$in": user_ids}, "GuildId": guild_id};
        let mut cursor = linked_users.find(filter, None).await?;
        let mut result = HashMap::<i64, RoGuildUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => {
                    let u = bson::from_document::<RoGuildUser>(document)?;
                    result.insert(u.discord_id, u);
                }
                Err(e) => tracing::error!(err = ?e),
            }
        }

        let users = self.get_users(user_ids).await?;
        for user in users {
            result.entry(user.discord_id).or_insert(RoGuildUser {
                discord_id: user.discord_id,
                guild_id: guild_id as i64,
                roblox_id: user.roblox_id,
            });
        }
        Ok(result.into_iter().map(|u| u.1).collect())
    }

    pub async fn add_backup(&self, mut backup: BackupGuild, name: &str) -> Result<()> {
        let backups = self.client.database("RoWifi").collection("backups");
        match self.get_backup(backup.user_id as u64, name).await? {
            Some(b) => {
                backup.id = b.id;
                let backup_bson = bson::to_bson(&backup)?;
                if let Bson::Document(b) = backup_bson {
                    let _res = backups
                        .find_one_and_replace(
                            doc! {"UserId": backup.user_id, "Name": backup.name},
                            b,
                            FindOneAndReplaceOptions::default(),
                        )
                        .await?;
                }
            }
            None => {
                let backup_bson = bson::to_bson(&backup)?;
                if let Bson::Document(b) = backup_bson {
                    let _res = backups.insert_one(b, InsertOneOptions::default()).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_backup(&self, user_id: u64, name: &str) -> Result<Option<BackupGuild>> {
        let backups = self.client.database("RoWifi").collection("backups");
        let filter = doc! {"UserId": user_id, "Name": name};
        let result = backups.find_one(filter, FindOneOptions::default()).await?;
        match result {
            Some(b) => Ok(Some(bson::from_bson::<BackupGuild>(Bson::Document(b))?)),
            None => Ok(None),
        }
    }

    pub async fn get_backups(&self, user_id: u64) -> Result<Vec<BackupGuild>> {
        let backups = self.client.database("RoWifi").collection("backups");
        let filter = doc! {"UserId": user_id};
        let mut cursor = backups.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<BackupGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => {
                    result.push(bson::from_bson::<BackupGuild>(Bson::Document(document))?)
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(result)
    }

    pub async fn get_premium(&self, user_id: u64) -> Result<Option<PremiumUser>> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let filter = doc! {"_id": user_id};
        let result = premium.find_one(filter, FindOneOptions::default()).await?;
        match result {
            Some(p) => Ok(Some(bson::from_bson::<PremiumUser>(Bson::Document(p))?)),
            None => Ok(None),
        }
    }

    pub async fn get_transferred_premium(&self, user_id: u64) -> Result<Option<PremiumUser>> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let filter = doc! {"PremiumOwner": user_id};
        let result = premium.find_one(filter, FindOneOptions::default()).await?;
        match result {
            Some(p) => Ok(Some(bson::from_bson::<PremiumUser>(Bson::Document(p))?)),
            None => Ok(None),
        }
    }

    pub async fn add_premium(
        &self,
        premium_user: PremiumUser,
        premium_already: bool,
    ) -> Result<()> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let premium_doc = bson::to_bson(&premium_user)?;
        if let Bson::Document(p) = premium_doc {
            if premium_already {
                let _res = premium
                    .find_one_and_replace(
                        doc! {"_id": premium_user.discord_id},
                        p,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _res = premium.insert_one(p, InsertOneOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn modify_premium(&self, filter: Document, update: Document) -> Result<()> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let _res = premium
            .find_one_and_update(filter, update, FindOneAndUpdateOptions::default())
            .await?;
        Ok(())
    }

    pub async fn delete_premium(&self, user_id: u64) -> Result<()> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let res = premium
            .find_one_and_delete(doc! {"_id": user_id}, FindOneAndDeleteOptions::default())
            .await?;
        if let Some(doc) = res {
            let premium_user = bson::from_bson::<PremiumUser>(Bson::Document(doc))?;
            for s in premium_user.discord_servers {
                let filter = bson::doc! {"_id": s};
                let update = bson::doc! {"$set": {"Settings.Type": GuildType::Normal as i32, "Settings.AutoDetection": false}};
                self.modify_guild(filter, update).await?;
            }
        }
        Ok(())
    }

    pub async fn get_all_premium(&self) -> Result<Vec<PremiumUser>> {
        let premium = self.client.database("RoWifi").collection("premium_new");
        let mut cursor = premium.find(None, None).await?;
        let mut docs = Vec::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => {
                    let premium_user = bson::from_bson::<PremiumUser>(Bson::Document(document))?;
                    docs.push(premium_user);
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(docs)
    }

    pub async fn get_analytics_membercount(&self, filter: Document) -> Result<Vec<Group>> {
        let member_count = self.client.database("Analytics").collection("member_count");
        let mut cursor = member_count.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<Group>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(ref document) => {
                    let doc = Bson::Document(document.clone());
                    match bson::from_bson::<Group>(doc) {
                        Ok(group) => result.push(group),
                        Err(e) => {
                            tracing::error!(error = ?e, doc = ?document, "Error in deserializing")
                        }
                    }
                }
                Err(e) => tracing::error!(error = ?e, "Error in the cursor"),
            }
        }
        Ok(result)
    }

    pub async fn add_event(&self, guild_id: u64, event_log: &EventLog) -> Result<()> {
        let events = self.client.database("Events").collection("Logs");

        let event_log_doc = bson::to_bson(event_log)?;
        if let Bson::Document(doc) = event_log_doc {
            events.insert_one(doc, None).await?;
        }

        let filter = doc! {"_id": guild_id};
        let update = doc! {"$inc": {"EventCounter": 1}};
        self.modify_guild(filter, update).await?;
        Ok(())
    }

    pub async fn get_events(&self, pipeline: Vec<Document>) -> Result<Vec<EventLog>> {
        let events_attendees_collection = self.client.database("Events").collection("Logs");
        let mut cursor = events_attendees_collection
            .aggregate(pipeline, None)
            .await?;
        let mut result = Vec::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => {
                    let doc = Bson::Document(document);
                    match bson::from_bson::<EventLog>(doc) {
                        Ok(event) => result.push(event),
                        Err(e) => {
                            tracing::error!(error = ?e, "Error in deserializing")
                        }
                    }
                }
                Err(err) => tracing::error!(err = ?err, "Error in cursor"),
            }
        }
        Ok(result)
    }
}
