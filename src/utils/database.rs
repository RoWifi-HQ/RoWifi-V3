use bson::{doc, Bson::Document};
use mongodb::{Client, options::*};
use futures::stream::StreamExt;
use crate::models::{guild::RoGuild, user::*};
use super::error::RoError;

pub struct Database {
    client: Client
}

impl Database {
    pub async fn new(conn_string: &str) -> Self {
        let client_options = ClientOptions::parse(conn_string).await.unwrap();
        let client = Client::with_options(client_options).unwrap();
        Self {
            client
        }
    }

    pub async fn get_guild(&self, guild_id: &u64) -> Result<Option<RoGuild>, RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let result = guilds.find_one(doc! {"_id": guild_id}, FindOneOptions::default()).await?;
        match result {
            None => Ok(None),
            Some(res) => Ok(Some(bson::from_bson::<RoGuild>(Document(res))?))
        }
    }

    pub async fn get_guilds(&self, guild_ids: Vec<u64>, premium_only: bool) -> Result<Vec<RoGuild>, RoError> {
        let guilds = self.client.database("RoWifi").collection("guilds");
        let filter = match premium_only {
            true => doc! {"Settings.AutoDetection": true, "_id": {"$in": guild_ids}},
            false => doc! {"_id": {"$in": guild_ids}}
        };
        let mut cursor = guilds.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoGuild>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<RoGuild>(Document(document))?),
                Err(e) => return Err(e.into())
            }
        }
        Ok(result)
    }

    pub async fn add_queue_user(&self, user: QueueUser) -> Result<(), RoError> {
        let queue = self.client.database("RoWifi").collection("queue");

        let exists = queue.find_one(doc! {"_id": user.roblox_id}, FindOneOptions::default()).await?.is_some();

        let user_doc = bson::to_bson(&user)?;
        if let Document(u) = user_doc {
            if exists {
                let _ = queue.find_one_and_replace(doc! {"_id": user.roblox_id}, u, FindOneAndReplaceOptions::default()).await?;
            } else {
                let _ = queue.insert_one(u, InsertOneOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn add_user(&self, user: RoUser, verified: bool) -> Result<(), RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let user_doc = bson::to_bson(&user)?;
        if let Document(u) = user_doc {
            if !verified {
                let _ = users.insert_one(u, InsertOneOptions::default()).await?;
            } else {
                let _ = users.find_one_and_replace(doc! {"_id": user.discord_id}, u, FindOneAndReplaceOptions::default()).await?;
            }
        }
        Ok(())
    }

    pub async fn get_user(&self, user_id: u64) -> Result<Option<RoUser>, RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let result = users.find_one(doc! {"_id": user_id}, FindOneOptions::default()).await?;
        match result {
            None => Ok(None),
            Some(res) => Ok(Some(bson::from_bson::<RoUser>(Document(res))?))
        }
    }

    pub async fn get_users(&self, user_ids: Vec<&u64>) -> Result<Vec<RoUser>, RoError> {
        let users = self.client.database("RoWifi").collection("users");
        let filter = doc! {"_id": {"$in": user_ids}};
        let mut cursor = users.find(filter, FindOptions::default()).await?;
        let mut result = Vec::<RoUser>::new();
        while let Some(res) = cursor.next().await {
            match res {
                Ok(document) => result.push(bson::from_bson::<RoUser>(Document(document))?),
                Err(e) => return Err(e.into())
            }
        }
        Ok(result)
    }   
}