#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use std::ops::Deref;

use async_trait::async_trait;
use deadpool::managed::{Manager, Object, Pool, RecycleResult};
use redis::{
    aio::{Connection as RedisAIOConnection, ConnectionLike},
    Client as RedisClient, Cmd, IntoConnectionInfo, Pipeline, RedisError, RedisFuture, RedisResult,
    Value,
};

pub use deadpool::managed::PoolError;
pub use redis;

pub type RedisPool = Pool<RedisManager, RedisConnection>;

pub struct RedisConnection {
    conn: Object<RedisManager>,
}

impl RedisConnection {
    #[must_use]
    pub fn take(this: Self) -> RedisAIOConnection {
        Object::<RedisManager>::take(this.conn)
    }
}

impl Deref for RedisConnection {
    type Target = RedisAIOConnection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl From<Object<RedisManager>> for RedisConnection {
    fn from(conn: Object<RedisManager>) -> Self {
        Self { conn }
    }
}

impl ConnectionLike for RedisConnection {
    fn req_packed_command<'a>(&'a mut self, cmd: &'a Cmd) -> RedisFuture<'a, Value> {
        self.conn.req_packed_command(cmd)
    }

    fn req_packed_commands<'a>(
        &'a mut self,
        cmd: &'a Pipeline,
        offset: usize,
        count: usize,
    ) -> RedisFuture<'a, Vec<Value>> {
        self.conn.req_packed_commands(cmd, offset, count)
    }

    fn get_db(&self) -> i64 {
        self.conn.get_db()
    }
}

pub struct RedisManager {
    client: RedisClient,
}

impl RedisManager {
    pub fn new<T: IntoConnectionInfo>(params: T) -> RedisResult<Self> {
        Ok(Self {
            client: RedisClient::open(params)?,
        })
    }
}

#[async_trait]
impl Manager for RedisManager {
    type Type = RedisAIOConnection;
    type Error = RedisError;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(self.client.get_async_connection().await?)
    }

    async fn recycle(&self, conn: &mut Self::Type) -> RecycleResult<Self::Error> {
        match redis::cmd("PING").query_async::<_, Value>(conn).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
