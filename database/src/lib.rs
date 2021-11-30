pub mod error;

use deadpool_postgres::{Manager, Object, Pool, Runtime};
use itertools::Itertools;
use rowifi_models::{FromRow, guild::RoGuild, user::{RoGuildUser, RoUser}};
use rustls::{ClientConfig as RustlsConfig, OwnedTrustAnchor, RootCertStore};
use rustls_pemfile::certs;
use std::{fs::File, io::BufReader, str::FromStr, time::Duration};
use tokio_postgres::{types::ToSql, Config as TokioPostgresConfig};
use tokio_postgres_rustls::MakeRustlsConnect;

use error::DatabaseError;

pub use tokio_postgres as postgres;

pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(connection_string: &str) -> Self {
        let postgres_config = TokioPostgresConfig::from_str(connection_string).unwrap();
        let mut cert_store = RootCertStore::empty();

        cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let cert_file = File::open("ca-certificates/us-east-1-bundle.pem").unwrap();
        let mut buf = BufReader::new(cert_file);
        let certs = certs(&mut buf).unwrap();
        cert_store.add_parsable_certificates(&certs);

        let rustls_config = RustlsConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(cert_store)
            .with_no_client_auth();

        let tls = MakeRustlsConnect::new(rustls_config);
        let manager = Manager::new(postgres_config, tls);
        let pool = Pool::builder(manager)
            .runtime(Runtime::Tokio1)
            .recycle_timeout(Some(Duration::from_secs(30)))
            .wait_timeout(Some(Duration::from_secs(30)))
            .create_timeout(Some(Duration::from_secs(30)))
            .build()
            .unwrap();

        tracing::debug!("Connecting to postgres...");
        let _ = pool.get().await.unwrap();

        Self { pool }
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

    pub async fn query_one<T: FromRow>(&self, statement: &str, params: &[&(dyn ToSql + Sync)]) -> Result<T, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached(statement).await?;
        let row = client.query_one(&statement, params).await?;
        Ok(T::from_row(row)?)
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

    pub async fn get_guild(&self, guild_id: i64) -> Result<RoGuild, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached("SELECT * FROM guilds WHERE guild_id = $1").await?;
        let row = client.query_opt(&statement, &[&guild_id]).await?;
        if let Some(row) = row {
            RoGuild::from_row(row).map_err(|e| e.into())
        } else {
            let guild = RoGuild::new(guild_id);
            let statement = client.prepare_cached(
                "INSERT INTO guilds(guild_id, command_prefix, kind, blacklist_action) VALUES($1, $2, $3, $4)",
            ).await?;
            client.execute(&statement, &[&guild_id, &guild.command_prefix, &guild.kind, &guild.blacklist_action]).await?;
            Ok(guild)
        }
    }

    pub async fn get_user(&self, user_id: i64) -> Result<Option<RoUser>, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached("SELECT * FROM users WHERE discord_id = $1").await?;
        let row = client.query_opt(&statement, &[&user_id]).await?;
        if let Some(row) = row {
            Ok(Some(RoUser::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_linked_user(&self, user_id: i64, guild_id: i64) -> Result<Option<RoGuildUser>, DatabaseError> {
        let client = self.get().await?;
        let statement = client.prepare_cached("SELECT * FROM linked_users WHERE guild_id = $1 AND discord_id = $2").await?;
        let row = client.query_opt(&statement, &[&guild_id, &user_id]).await?;
        if let Some(row) = row {
            Ok(Some(RoGuildUser::from_row(row)?))
        } else {
            let statement = client.prepare_cached("SELECT * FROM users WHERE discord_id = $1").await?;
            let row = client.query_opt(&statement, &[&user_id]).await?;
            if let Some(row) = row {
                let user = RoUser::from_row(row)?;
                Ok(Some(RoGuildUser {
                    guild_id,
                    discord_id: user_id,
                    roblox_id: user.default_roblox_id
                }))
            } else {
                Ok(None)
            }
        }
    }
}

pub fn dynamic_args(size: usize) -> String {
    (0..size).map(|i| format!("${}", i+1)).join(", ")
}
