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

use futures_util::stream::StreamExt;
use mongodb::{
    bson::{self, doc, document::Document},
    options::{ClientOptions, FindOneAndReplaceOptions, FindOneAndUpdateOptions, ReturnDocument},
    Client,
};
use rowifi_models::{
    analytics::Group,
    events::EventLog,
    guild::{BackupGuild, GuildType, RoGuild},
    user::{PremiumUser, QueueUser, RoGuildUser, RoUser},
};
use rowifi_redis::{redis::AsyncCommands, RedisPool};
use std::{collections::HashMap, result::Result as StdResult};

pub use error::DatabaseError;

type Result<T> = StdResult<T, DatabaseError>;

const DATABASE: &str = "RoWifi";
const GUILDS: &str = "guilds";
const QUEUE: &str = "queue";
const USERS: &str = "users";
const LINKED_USERS: &str = "linked_users";
const PREMIUM: &str = "premium_new";

#[derive(Clone)]
pub struct Database {
    client: Client,
    redis_pool: RedisPool,
}

impl Database {
    /// Create a database instance by passing in the connection string and redis pool object
    pub async fn new(conn_string: &str, redis_pool: RedisPool) -> Self {
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        Self { client, redis_pool }
    }

    /// Add or replace a guild in the database
    pub async fn add_guild(&self, guild: RoGuild, replace: bool) -> Result<()> {
        let guilds = self.client.database(DATABASE).collection(GUILDS);
        let guild_doc = bson::to_document(&guild)?;
        let key = format!("database:g:{}", guild.id);
        let mut conn = self.redis_pool.get().await?;
        if replace {
            let mut options = FindOneAndReplaceOptions::default();
            options.return_document = Some(ReturnDocument::After);
            let res = guilds
                .find_one_and_replace(doc! {"_id": guild.id}, guild_doc, options)
                .await?;
            if let Some(res) = res {
                let guild = bson::from_document::<RoGuild>(res)?;
                let _: () = conn.set_ex(key, guild, 6 * 3600).await.unwrap();
            }
        } else {
            let _res = guilds.insert_one(guild_doc, None).await?;
            let _: () = conn.set_ex(key, guild, 6 * 3600).await.unwrap();
        }
        Ok(())
    }

    /// Get the guild from its id. If it's not present in the cache, it will be brought from the database and stored in the cache.
    pub async fn get_guild(&self, guild_id: u64) -> Result<Option<RoGuild>> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:g:{}", guild_id);
        let guild: Option<RoGuild> = conn.get(&key).await?;
        match guild {
            Some(g) => Ok(Some(g)),
            None => {
                let guilds = self.client.database(DATABASE).collection(GUILDS);
                let result = guilds.find_one(doc! {"_id": guild_id}, None).await?;
                let guild = match result {
                    None => return Ok(None),
                    Some(res) => bson::from_document::<RoGuild>(res)?,
                };
                let _: () = conn.set_ex(key, guild.clone(), 6 * 3600).await?;
                Ok(Some(guild))
            }
        }
    }

    /// Get multiple guilds from their ids. This method bypasses the cache
    pub async fn get_guilds(&self, guild_ids: &[u64], premium_only: bool) -> Result<Vec<RoGuild>> {
        let guilds = self
            .client
            .database(DATABASE)
            .collection::<Document>(GUILDS);
        let filter = if premium_only {
            doc! {"Settings.AutoDetection": true, "_id": {"$in": guild_ids}}
        } else {
            doc! {"_id": {"$in": guild_ids}}
        };
        let mut cursor = guilds.find(filter, None).await?;
        let mut result = Vec::<RoGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => match bson::from_document::<RoGuild>(document.clone()) {
                    Ok(guild) => result.push(guild),
                    Err(e) => {
                        tracing::error!(error = ?e, doc = ?document, "Error in deserializing")
                    }
                },
                Err(e) => tracing::error!(error = ?e, "Error in the cursor"),
            }
        }
        Ok(result)
    }

    /// Modify the guild in the database and store the updated result in the cache
    pub async fn modify_guild(&self, filter: Document, update: Document) -> Result<()> {
        let guilds = self.client.database(DATABASE).collection(GUILDS);
        let mut conn = self.redis_pool.get().await?;

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();
        let res = guilds
            .find_one_and_update(filter, update, options)
            .await?
            .unwrap();
        let guild = bson::from_document::<RoGuild>(res)?;

        let key = format!("database:g:{}", guild.id);
        let _: () = conn.set_ex(key, guild, 6 * 3600).await?;
        Ok(())
    }

    /// Add an user who's currently initiated a verification prompt
    pub async fn add_queue_user(&self, user: QueueUser) -> Result<()> {
        let queue = self.client.database(DATABASE).collection(QUEUE);
        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:u:{}", user.discord_id);

        let exists = queue
            .find_one(doc! {"_id": user.roblox_id}, None)
            .await?
            .is_some();
        let user_doc = bson::to_document(&user)?;
        if exists {
            let _res = queue
                .find_one_and_replace(doc! {"_id": user.roblox_id}, user_doc, None)
                .await?;
        } else {
            let _res = queue.insert_one(user_doc, None).await?;
        }

        let _: () = conn.del(key).await?;
        Ok(())
    }

    /// Add or replace the user in the database and store the result in the cache
    pub async fn add_user(&self, user: RoUser, verified: bool) -> Result<()> {
        let users = self.client.database(DATABASE).collection(USERS);
        let user_doc = bson::to_document(&user)?;
        let mut conn = self.redis_pool.get().await?;

        if verified {
            let _res = users
                .find_one_and_replace(doc! {"_id": user.discord_id}, user_doc, None)
                .await?;
        } else {
            let _res = users.insert_one(user_doc, None).await?;
        }

        let key = format!("database:u:{}", user.discord_id);
        let _: () = conn.set_ex(key, user, 6 * 3600).await?;
        Ok(())
    }

    /// Add a user to the `linked_user` collection
    pub async fn add_linked_user(&self, linked_user: RoGuildUser) -> Result<()> {
        let linked_users = self.client.database(DATABASE).collection(LINKED_USERS);

        let mut conn = self.redis_pool.get().await?;
        let key = format!(
            "database:l:{}:{}",
            linked_user.discord_id, linked_user.guild_id
        );

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

        let _: () = conn.set_ex(key, linked_user, 6 * 3600).await?;
        Ok(())
    }

    /// Delete all documents of a discord user with the given roblox id. This method is called after an user runs `verify delete`
    pub async fn delete_linked_users(&self, user_id: u64, roblox_id: i64) -> Result<()> {
        let linked_users = self.client.database(DATABASE).collection(LINKED_USERS);
        let filter = doc! {"UserId": user_id, "RobloxId": roblox_id};
        let mut conn = self.redis_pool.get().await?;

        let mut cursor = linked_users.find(filter.clone(), None).await?;
        let mut result = Vec::<RoGuildUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_document::<RoGuildUser>(document)?),
                Err(e) => tracing::error!(err = ?e),
            }
        }

        for lu in result {
            let key = format!("database:l:{}:{}", lu.discord_id, lu.guild_id);
            let _: () = conn.del(key).await?;
        }

        let _res = linked_users.delete_many(filter, None).await?;
        Ok(())
    }

    /// Get the user from the database
    pub async fn get_user(&self, user_id: u64) -> Result<Option<RoUser>> {
        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:u:{}", user_id);

        let user: Option<RoUser> = conn.get(&key).await?;
        match user {
            Some(u) => Ok(Some(u)),
            None => {
                let users = self.client.database(DATABASE).collection(USERS);
                let result = users.find_one(doc! {"_id": user_id}, None).await?;
                let user = match result {
                    None => return Ok(None),
                    Some(res) => bson::from_document::<RoUser>(res)?,
                };
                let _: () = conn.set_ex(key, user.clone(), 6 * 3600).await?;
                Ok(Some(user))
            }
        }
    }

    /// Get a user from the database with the given `guild_id`
    pub async fn get_linked_user(
        &self,
        user_id: u64,
        guild_id: u64,
    ) -> Result<Option<RoGuildUser>> {
        let linked_users = self.client.database(DATABASE).collection(LINKED_USERS);

        let mut conn = self.redis_pool.get().await?;
        let key = format!("database:l:{}:{}", user_id, guild_id);

        let linked_user: Option<RoGuildUser> = conn.get(&key).await?;
        match linked_user {
            Some(l) => Ok(Some(l)),
            None => {
                let result = linked_users
                    .find_one(doc! {"GuildId": guild_id, "UserId": user_id}, None)
                    .await?;
                let user = match result {
                    None => return Ok(None),
                    Some(doc) => bson::from_document::<RoGuildUser>(doc)?,
                };
                let _: () = conn.set_ex(key, user.clone(), 6 * 3600).await?;
                Ok(Some(user))
            }
        }
    }

    /// Get multiple users from provided ids. This method bypasses the cache
    pub async fn get_users(&self, user_ids: &[u64]) -> Result<Vec<RoUser>> {
        let users = self.client.database(DATABASE).collection(USERS);
        let filter = doc! {"_id": {"$in": user_ids}};
        let mut cursor = users.find(filter, None).await?;
        let mut result = Vec::<RoUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_document::<RoUser>(document)?),
                Err(e) => tracing::error!(err = ?e),
            }
        }
        Ok(result)
    }

    /// Get multiple users based on the `guild_id`
    pub async fn get_linked_users(
        &self,
        user_ids: &[u64],
        guild_id: u64,
    ) -> Result<Vec<RoGuildUser>> {
        let linked_users = self.client.database(DATABASE).collection(LINKED_USERS);
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

    /// Add a backup to the database with the given name
    pub async fn add_backup(&self, mut backup: BackupGuild, name: &str) -> Result<()> {
        let backups = self.client.database(DATABASE).collection("backups");
        match self.get_backup(backup.user_id as u64, name).await? {
            Some(b) => {
                backup.id = b.id;
                let backup_doc = bson::to_document(&backup)?;
                let _res = backups
                    .find_one_and_replace(
                        doc! {"UserId": backup.user_id, "Name": backup.name},
                        backup_doc,
                        None,
                    )
                    .await?;
            }
            None => {
                let backup_doc = bson::to_document(&backup)?;
                let _res = backups.insert_one(backup_doc, None).await?;
            }
        }
        Ok(())
    }

    /// Get a backup provided the `user_id` and `name`
    pub async fn get_backup(&self, user_id: u64, name: &str) -> Result<Option<BackupGuild>> {
        let backups = self.client.database(DATABASE).collection("backups");
        let filter = doc! {"UserId": user_id, "Name": name};
        let result = backups.find_one(filter, None).await?;
        match result {
            Some(b) => Ok(Some(bson::from_document::<BackupGuild>(b)?)),
            None => Ok(None),
        }
    }

    /// Get all the backups of a certain user
    pub async fn get_backups(&self, user_id: u64) -> Result<Vec<BackupGuild>> {
        let backups = self.client.database(DATABASE).collection("backups");
        let filter = doc! {"UserId": user_id};
        let mut cursor = backups.find(filter, None).await?;
        let mut result = Vec::<BackupGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_document::<BackupGuild>(document)?),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(result)
    }

    /// Get the premium information about an user
    pub async fn get_premium(&self, user_id: u64) -> Result<Option<PremiumUser>> {
        let premium = self.client.database(DATABASE).collection(PREMIUM);
        let filter = doc! {"_id": user_id};
        let result = premium.find_one(filter, None).await?;
        match result {
            Some(p) => Ok(Some(bson::from_document::<PremiumUser>(p)?)),
            None => Ok(None),
        }
    }

    /// Get the premium information about a user who has premium transferred to them provided the premium owner's id
    pub async fn get_transferred_premium(&self, user_id: u64) -> Result<Option<PremiumUser>> {
        let premium = self.client.database(DATABASE).collection(PREMIUM);
        let filter = doc! {"PremiumOwner": user_id};
        let result = premium.find_one(filter, None).await?;
        match result {
            Some(p) => Ok(Some(bson::from_document::<PremiumUser>(p)?)),
            None => Ok(None),
        }
    }

    /// Add or replace premium of an user in the database
    pub async fn add_premium(
        &self,
        premium_user: PremiumUser,
        premium_already: bool,
    ) -> Result<()> {
        let premium = self.client.database(DATABASE).collection(PREMIUM);
        let premium_doc = bson::to_document(&premium_user)?;
        if premium_already {
            let _res = premium
                .find_one_and_replace(doc! {"_id": premium_user.discord_id}, premium_doc, None)
                .await?;
        } else {
            let _res = premium.insert_one(premium_doc, None).await?;
        }
        Ok(())
    }

    /// Modify the premium of an user
    pub async fn modify_premium(&self, filter: Document, update: Document) -> Result<()> {
        let premium = self
            .client
            .database(DATABASE)
            .collection::<Document>(PREMIUM);
        let _res = premium.find_one_and_update(filter, update, None).await?;
        Ok(())
    }

    /// Delete the premium of an user
    pub async fn delete_premium(&self, user_id: u64) -> Result<()> {
        let premium = self.client.database(DATABASE).collection(PREMIUM);
        let res = premium
            .find_one_and_delete(doc! {"_id": user_id}, None)
            .await?;
        if let Some(doc) = res {
            let premium_user = bson::from_document::<PremiumUser>(doc)?;
            for s in premium_user.discord_servers {
                let filter = bson::doc! {"_id": s};
                let update = bson::doc! {"$set": {"Settings.Type": GuildType::Normal as i32, "Settings.AutoDetection": false}};
                self.modify_guild(filter, update).await?;
            }
        }
        Ok(())
    }

    /// Get all premium users from the database
    pub async fn get_all_premium(&self) -> Result<Vec<PremiumUser>> {
        let premium = self.client.database(DATABASE).collection(PREMIUM);
        let mut cursor = premium.find(None, None).await?;
        let mut docs = Vec::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => {
                    let premium_user = bson::from_document::<PremiumUser>(document)?;
                    docs.push(premium_user);
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(docs)
    }

    /// Get the time series of member counts of a group
    pub async fn get_analytics_membercount(&self, filter: Document) -> Result<Vec<Group>> {
        let member_count = self
            .client
            .database("Analytics")
            .collection::<Document>("member_count");
        let mut cursor = member_count.find(filter, None).await?;
        let mut result = Vec::<Group>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => match bson::from_document::<Group>(document.clone()) {
                    Ok(group) => result.push(group),
                    Err(e) => {
                        tracing::error!(error = ?e, doc = ?document, "Error in deserializing")
                    }
                },
                Err(e) => tracing::error!(error = ?e, "Error in the cursor"),
            }
        }
        Ok(result)
    }

    /// Log an event of a guild in the database
    pub async fn add_event(&self, guild_id: u64, event_log: &EventLog) -> Result<()> {
        let events = self.client.database("Events").collection("Logs");

        let event_log_doc = bson::to_document(event_log)?;
        events.insert_one(event_log_doc, None).await?;

        let filter = doc! {"_id": guild_id};
        let update = doc! {"$inc": {"EventCounter": 1}};
        self.modify_guild(filter, update).await?;
        Ok(())
    }

    /// Get events of a guild based on the filter criteria in `pipeline`
    pub async fn get_events(&self, pipeline: Vec<Document>) -> Result<Vec<EventLog>> {
        let events_attendees_collection = self
            .client
            .database("Events")
            .collection::<Document>("Logs");
        let mut cursor = events_attendees_collection
            .aggregate(pipeline, None)
            .await?;
        let mut result = Vec::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => match bson::from_document::<EventLog>(document) {
                    Ok(event) => result.push(event),
                    Err(e) => {
                        tracing::error!(error = ?e, "Error in deserializing")
                    }
                },
                Err(err) => tracing::error!(err = ?err, "Error in cursor"),
            }
        }
        Ok(result)
    }
}

impl AsRef<Client> for Database {
    fn as_ref(&self) -> &Client {
        &self.client
    }
}
