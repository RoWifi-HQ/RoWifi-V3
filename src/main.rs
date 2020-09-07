#![allow(dead_code)]

mod cache;
mod commands;
mod framework;
mod models;
mod rolang;
mod services;
mod utils;

use std::{env, error::Error, sync::Arc};
use tokio::stream::StreamExt;
use twilight_gateway::cluster::{ShardScheme, Cluster};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_standby::Standby;

use cache::Cache;
use commands::*;
use framework::{context::Context, Framework};
use services::*;
use utils::{Database, Roblox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    kankyo::load().err();
    tracing_subscriber::fmt::init();
    //tracing_log::LogTracer::init()?;

    let token = env::var("DISC_TOKEN")?;
    let conn_string = env::var("DB_CONN")?;
    let scheme = ShardScheme::Auto;
    let http = HttpClient::new(&token);

    let cluster = Cluster::builder(&token)
        .shard_scheme(scheme)
        .intents(Some(
            Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_MESSAGE_REACTIONS
        ))
        .http_client(http.clone())
        .build().await?;

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let cache = Arc::new(Cache::new());
    let standby = Standby::new();

    let database = Database::new(&conn_string).await;
    let roblox = Roblox::new();

    let context = Context::new(0, http, cache, database, roblox, standby, cluster);
    let framework = Framework::default()
        .configure(|c| c
            .default_prefix("?")
        )
        .command(&UPDATE_COMMAND)
        .command(&VERIFY_COMMAND)
        .command(&REVERIFY_COMMAND)
        .command(&RANKBINDS_COMMAND)
        .command(&GROUPBINDS_COMMAND)
        .command(&CUSTOMBINDS_COMMAND)
        .command(&ASSETBINDS_COMMAND)
        .command(&BLACKLISTS_COMMAND)
        .command(&SETTINGS_COMMAND)
        .command(&SETUP_COMMAND);

    let framework = Arc::new(Box::new(framework));
    let event_handler = EventHandler::default();

    let context_ad = context.clone();
    tokio::spawn(async move{
        let _ = auto_detection(context_ad).await;
    });

    let mut events = context.cluster.events();
    while let Some(event) = events.next().await {
        let c = context.clone();
        let f = Arc::clone(&framework);
        let e = event_handler.clone();
        context.cache.update(&event.1).expect("Failed to update cache");
        context.standby.process(&event.1);
        
        tokio::spawn(async move {
            e.handle_event(event.0, &event.1, c.clone()).await;
            f.handle_event(event.1, c).await;
        });
    }

    Ok(())
}
