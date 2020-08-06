use std::sync::Arc;
use twilight::{
    http::Client as Http,
    cache::InMemoryCache as Cache
};

use crate::utils::{Database, Roblox};

#[derive(Clone)]
pub struct Context {
    pub shard_id: u64,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
    pub database: Arc<Database>,
    pub roblox: Arc<Roblox>
}

impl Context {
    pub fn new(shard_id: u64, http: Arc<Http>, cache: Arc<Cache>, database: Arc<Database>, roblox: Arc<Roblox>) -> Self {
        Self {
            shard_id,
            http,
            cache,
            database,
            roblox
        }
    } 
}