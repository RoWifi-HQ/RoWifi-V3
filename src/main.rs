#![allow(dead_code)]

mod cache;
mod commands;
mod framework;
mod models;
mod rolang;
mod utils;

use std::{env, error::Error, sync::Arc};
use tokio::stream::StreamExt;
use twilight_gateway::cluster::{ShardScheme, Cluster};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::GatewayIntents;
use twilight_standby::Standby;

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
    let http = HttpClient::new(&token);

    let cluster = Cluster::builder(&token)
        .shard_scheme(scheme)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILD_MESSAGE_REACTIONS
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

    let context = Context::new(0, http, cache, database, roblox, standby);
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
        .command(&ASSETBINDS_COMMAND);

    let framework = Arc::new(Box::new(framework));

    let mut events = cluster.events();
    while let Some(event) = events.next().await {
        let c = context.clone();
        let f = Arc::clone(&framework);
        context.cache.update(&event.1).expect("Failed to update cache");
        context.standby.process(&event.1);
        tokio::spawn(async move {
            f.handle_event(event.1, c).await;
        });
    }

    Ok(())
}
