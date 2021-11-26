pub mod error;

use deadpool_postgres::{Manager, Object, Pool, Runtime};
use itertools::Itertools;
use rowifi_models::FromRow;
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
}

pub fn dynamic_args(size: usize) -> String {
    (0..size).map(|i| format!("${}", i+1)).join(", ")
}
