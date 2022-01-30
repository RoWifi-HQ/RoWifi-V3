pub mod error;

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead},
    Aes256Gcm, Key, NewAead,
};
use deadpool_postgres::{Manager, Object, Pool, Runtime};
use itertools::Itertools;
use rowifi_models::{
    guild::RoGuild,
    id::{GuildId, UserId},
    user::{RoGuildUser, RoUser},
    FromRow,
};
use std::{str::FromStr, time::Duration};
use tokio_postgres::{types::ToSql, Config as TokioPostgresConfig, NoTls};

use error::DatabaseError;

pub use tokio_postgres as postgres;

pub struct Database {
    pool: Pool,
    pub cipher: Aes256Gcm,
}

impl Database {
    pub async fn new(connection_string: &str, primary_key: &str) -> Self {
        let postgres_config = TokioPostgresConfig::from_str(connection_string).unwrap();

        let manager = Manager::new(postgres_config, NoTls);
        let pool = Pool::builder(manager)
            .runtime(Runtime::Tokio1)
            .recycle_timeout(Some(Duration::from_secs(30)))
            .wait_timeout(Some(Duration::from_secs(30)))
            .create_timeout(Some(Duration::from_secs(30)))
            .build()
            .unwrap();

        tracing::debug!("Connecting to postgres...");
        let _ = pool.get().await.unwrap();

        let key = Key::from_slice(primary_key.as_bytes());
        let cipher = Aes256Gcm::new(key);

        Self { pool, cipher }
    }

    pub async fn get(&self) -> Result<Object, DatabaseError> {
        let client = self.pool.get().await?;
        Ok(client)
    }

    pub async fn query<T: FromRow>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached(statement).await?;
        let rows = client.query(&statement, params).await?;
        let items = rows
            .into_iter()
            .map(|r| T::from_row(r))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    pub async fn query_one<T: FromRow>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached(statement).await?;
        let row = client.query_one(&statement, params).await?;
        Ok(T::from_row(row)?)
    }

    pub async fn query_opt<T: FromRow>(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<T>, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached(statement).await?;
        let row = client.query_opt(&statement, params).await?;
        match row {
            Some(r) => Ok(Some(T::from_row(r)?)),
            None => Ok(None),
        }
    }

    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<(), DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached(statement).await?;
        client.execute(&statement, params).await?;
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: GuildId) -> Result<RoGuild, DatabaseError> {
        let client = self.get().await?;
        let statement = client
            .prepare_cached("SELECT * FROM guilds WHERE guild_id = $1")
            .await?;
        let row = client.query_opt(&statement, &[&guild_id]).await?;
        if let Some(row) = row {
            RoGuild::from_row(row).map_err(|e| e.into())
        } else {
            let guild = RoGuild::new(guild_id);
            let statement = client.prepare_cached(
                "INSERT INTO guilds(guild_id, command_prefix, kind, blacklist_action) VALUES($1, $2, $3, $4)",
            ).await?;
            client
                .execute(
                    &statement,
                    &[
                        &guild_id,
                        &guild.command_prefix,
                        &guild.kind,
                        &guild.blacklist_action,
                    ],
                )
                .await?;
            Ok(guild)
        }
    }

    pub async fn get_user(&self, user_id: i64) -> Result<Option<RoUser>, DatabaseError> {
        let client = self.get().await?;
        let statement = client
            .prepare_cached("SELECT * FROM users WHERE discord_id = $1")
            .await?;
        let row = client.query_opt(&statement, &[&user_id]).await?;
        if let Some(row) = row {
            Ok(Some(RoUser::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_linked_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Option<RoGuildUser>, DatabaseError> {
        let client = self.get().await?;
        let statement = client
            .prepare_cached("SELECT * FROM linked_users WHERE guild_id = $1 AND discord_id = $2")
            .await?;
        let row = client.query_opt(&statement, &[&guild_id, &user_id]).await?;
        if let Some(row) = row {
            Ok(Some(RoGuildUser::from_row(row)?))
        } else {
            let statement = client
                .prepare_cached("SELECT * FROM users WHERE discord_id = $1")
                .await?;
            let row = client.query_opt(&statement, &[&user_id]).await?;
            if let Some(row) = row {
                let user = RoUser::from_row(row)?;
                Ok(Some(RoGuildUser {
                    guild_id,
                    discord_id: user_id,
                    roblox_id: user.default_roblox_id,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

#[inline]
pub fn dynamic_args(size: usize) -> String {
    (0..size).map(|i| format!("${}", i + 1)).join(", ")
}

#[inline]
pub fn dynamic_args_with_start(size: usize, start: usize) -> String {
    (0..size).map(|i| format!("${}", i + start)).join(", ")
}

pub fn encrypt_bytes(
    plaintext: &[u8],
    aaed: &Aes256Gcm,
    guild_id: u64,
    host_id: u64,
    timestamp: u64,
) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    let guild_id_bytes = guild_id.to_le_bytes();
    nonce[..4].copy_from_slice(&guild_id_bytes[4..]);
    let timestamp_bytes = timestamp.to_be_bytes();
    nonce[4..8].copy_from_slice(&timestamp_bytes[4..]);
    let host_id_bytes = host_id.to_be_bytes();
    nonce[8..].copy_from_slice(&host_id_bytes[4..]);

    let nonce = GenericArray::from_slice(&nonce);
    aaed.encrypt(nonce, plaintext).unwrap()
}

pub fn decrypt_bytes(
    ciphertext: &[u8],
    aaed: &Aes256Gcm,
    guild_id: u64,
    host_id: u64,
    timestamp: u64,
) -> Vec<u8> {
    let mut nonce = [0u8; 12];
    let guild_id_bytes = guild_id.to_le_bytes();
    nonce[..4].copy_from_slice(&guild_id_bytes[4..]);
    let timestamp_bytes = timestamp.to_be_bytes();
    nonce[4..8].copy_from_slice(&timestamp_bytes[4..]);
    let host_id_bytes = host_id.to_be_bytes();
    nonce[8..].copy_from_slice(&host_id_bytes[4..]);

    let nonce = GenericArray::from_slice(&nonce);
    aaed.decrypt(nonce, ciphertext).unwrap()
}
