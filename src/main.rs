#![allow(dead_code)]

mod cache;
mod commands;
mod framework;
mod models;
mod rolang;
mod utils;

use std::{env, error::Error, sync::Arc};
use tokio::stream::StreamExt;
use twilight::{
    gateway::cluster::{config::ShardScheme, Cluster, ClusterConfig},
    http::Client as HttpClient,
    model::gateway::GatewayIntents,
    standby::Standby
};

use cache::Cache;
use commands::*;
use framework::{context::Context, Framework};
use utils::{Database, Roblox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    kankyo::load().err();
    tracing_subscriber::fmt::init();
    //tracing_log::LogTracer::init()?;

    let token = env::var("DISC_TOKEN")?;
    let conn_string = env::var("DB_CONN")?;
    let scheme = ShardScheme::Auto;
    let http = Arc::new(HttpClient::new(&token));

    let config = ClusterConfig::builder(&token)
        .shard_scheme(scheme)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS
        ))
        .http_client(http.as_ref().clone())
        .build();

    let cluster = Cluster::new(config).await?;

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let cache = Arc::new(Cache::new());
    let standby = Arc::new(Standby::new());

    let database = Arc::new(Database::new(&conn_string).await);
    let roblox = Arc::new(Roblox::new());

    let context = Context::new(0, http, cache, database, roblox, standby);
    let framework = Framework::default()
        .configure(|c| c
            .default_prefix("?")
        )
        .command(&UPDATE_COMMAND)
        .command(&VERIFY_COMMAND);

    let framework = Arc::new(Box::new(framework));

    let mut events = cluster.events();
    while let Some(event) = events.next().await {
        let c = context.clone();
        let f = Arc::clone(&framework);
        context.cache.update(&event.1).await.expect("Failed to update cache");
        context.standby.process(&event.1);
        tokio::spawn(async move {
            f.handle_event(event.1, c).await;
        });
    }

    Ok(())
}
