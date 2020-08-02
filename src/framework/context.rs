use std::sync::Arc;
use tokio::sync::RwLock;
use twilight::http::Client as Http;
use typemap_rev::TypeMap;

#[derive(Clone)]
pub struct Context {
    pub data: Arc<RwLock<TypeMap>>,
    pub shard_id: u64,
    pub http: Arc<Http>
}

impl Context {
    pub fn new(data: Arc<RwLock<TypeMap>>, shard_id: u64, http: Arc<Http>) -> Self {
        Self {
            data,
            shard_id,
            http
        }
    }
}