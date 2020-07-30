use std::sync::Arc;
use tokio::sync::RwLock;
use twilight::http::Client as Http;
use typemap_rev::TypeMap;

pub struct Context {
    pub data: Arc<RwLock<TypeMap>>,
    pub shard_id: u64,
    pub http: Arc<Http>
}