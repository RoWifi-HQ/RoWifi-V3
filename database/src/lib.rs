#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::field_reassign_with_default,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::cast_possible_wrap,
    clippy::single_match_else,
    clippy::cast_sign_loss,
    clippy::missing_panics_doc
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
    user::{PremiumUser, QueueUser, RoUser},
};
use std::{result::Result as StdResult, sync::Arc, time::Duration};
use transient_dashmap::TransientDashMap;

pub use error::DatabaseError;

type Result<T> = StdResult<T, DatabaseError>;

#[derive(Clone)]
pub struct Database {
    client: Client,
    guild_cache: TransientDashMap<i64, Arc<RoGuild>>,
    user_cache: TransientDashMap<i64, Arc<RoUser>>,
}

impl Database {
    pub async fn new(conn_string: &str) -> Self {
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        Self {
            client,
            guild_cache: TransientDashMap::new(Duration::from_secs(6 * 3600)),
            user_cache: TransientDashMap::new(Duration::from_secs(6 * 3600)),
        }
    }

    pub async fn add_guild(&self, guild: RoGuild, replace: bool) -> Result<()> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let guild_bson = bson::to_bson(&guild)?;
        if let Bson::Document(g) = guild_bson {
            if replace {
                let mut options = FindOneAndReplaceOptions::default();
                options.return_document = Some(ReturnDocument::After);
                let res = guilds
                    .find_one_and_replace(doc! {"_id": guild.id}, g, options)
                    .await?;
                if let Some(res) = res {
                    let guild = Arc::new(bson::from_bson::<RoGuild>(Bson::Document(res))?);
                    self.guild_cache.insert(guild.id, guild);
                }
            } else {
                let _ = guilds.insert_one(g, InsertOneOptions::default()).await?;
                self.guild_cache.insert(guild.id, Arc::new(guild));
            }
        }
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: u64) -> Result<Option<Arc<RoGuild>>> {
        let guild_id = guild_id as i64;
        match self.guild_cache.get(&guild_id) {
            Some(g) => Ok(Some(g.value().object.clone())),
            None => {
                let guilds = self.client.database("RoWifi").collection("guilds");
                let result = guilds
                    .find_one(doc! {"_id": guild_id}, FindOneOptions::default())
                    .await?;
                let guild = match result {
                    None => return Ok(None),
                    Some(res) => Arc::new(bson::from_bson::<RoGuild>(Bson::Document(res))?),
                };
                self.guild_cache.insert(guild_id, guild.clone());
                Ok(Some(guild))
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
                    let doc = Bson::Document(document.to_owned());
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

        self.guild_cache.insert(guild.id, Arc::new(guild));
        Ok(())
    }

    pub async fn add_queue_user(&self, user: QueueUser) -> Result<()> {
        let queue = self.client.database("RoWifi").collection("queue");

        let exists = queue
            .find_one(doc! {"_id": user.roblox_id}, FindOneOptions::default())
            .await?
            .is_some();

        let user_doc = bson::to_bson(&user)?;
        if let Bson::Document(u) = user_doc {
            if exists {
                let _ = queue
                    .find_one_and_replace(
                        doc! {"_id": user.roblox_id},
                        u,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _ = queue.insert_one(u, InsertOneOptions::default()).await?;
            }
        }
        self.user_cache.remove(&user.discord_id);
        Ok(())
    }

    pub async fn add_user(&self, user: RoUser, verified: bool) -> Result<()> {
        let users = self.client.database("RoWifi").collection("users");
        let user_doc = bson::to_bson(&user)?;
        if let Bson::Document(u) = user_doc {
            if verified {
                let _ = users
                    .find_one_and_replace(
                        doc! {"_id": user.discord_id},
                        u,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _ = users.insert_one(u, InsertOneOptions::default()).await?;
            }
            self.user_cache.insert(user.discord_id, Arc::new(user));
        }
        Ok(())
    }

    pub async fn get_user(&self, user_id: u64) -> Result<Option<Arc<RoUser>>> {
        let user_id = user_id as i64;
        match self.user_cache.get(&user_id) {
            Some(u) => Ok(Some(u.value().object.clone())),
            None => {
                let users = self.client.database("RoWifi").collection("users");
                let result = users
                    .find_one(doc! {"_id": user_id}, FindOneOptions::default())
                    .await?;
                let user = match result {
                    None => return Ok(None),
                    Some(res) => Arc::new(bson::from_bson::<RoUser>(Bson::Document(res))?),
                };
                self.user_cache.insert(user_id, user.clone());
                Ok(Some(user))
            }
        }
    }

    pub async fn get_users(&self, user_ids: Vec<u64>) -> Result<Vec<RoUser>> {
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

    pub async fn add_backup(&self, mut backup: BackupGuild, name: &str) -> Result<()> {
        let backups = self.client.database("RoWifi").collection("backups");
        match self.get_backup(backup.user_id as u64, name).await? {
            Some(b) => {
                backup.id = b.id;
                let backup_bson = bson::to_bson(&backup)?;
                if let Bson::Document(b) = backup_bson {
                    let _ = backups
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
                    let _ = backups.insert_one(b, InsertOneOptions::default()).await?;
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
                let _ = premium
                    .find_one_and_replace(
                        doc! {"_id": premium_user.discord_id},
                        p,
                        FindOneAndReplaceOptions::default(),
                    )
                    .await?;
            } else {
                let _ = premium.insert_one(p, InsertOneOptions::default()).await?;
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
                    let doc = Bson::Document(document.to_owned());
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

    pub async fn add_event(&self, guild_id: i64, event_log: &EventLog) -> Result<()> {
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
