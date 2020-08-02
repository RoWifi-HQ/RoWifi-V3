mod framework;

use std::{env, error::Error};
use tokio::stream::StreamExt;
use twilight::{
    cache::{
        twilight_cache_inmemory::config::{InMemoryConfigBuilder, EventType},
        InMemoryCache,
    },
    gateway::{cluster::{config::ShardScheme, Cluster, ClusterConfig}, Event},
    http::Client as HttpClient,
    model::gateway::GatewayIntents,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    kankyo::load().err();
    tracing_subscriber::fmt::init();
    tracing_log::LogTracer::init()?;

    let token = env::var("DISC_TOKEN")?;
    let scheme = ShardScheme::Auto;
    let config = ClusterConfig::builder(&token)
        .shard_scheme(scheme)
        .intents(Some(
            GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS
        ))
        .build();

    let cluster = Cluster::new(config).await?;

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    let http = HttpClient::new(&token);
    let cache_config = InMemoryConfigBuilder::new()
        .event_types(
            EventType::MESSAGE_CREATE
                 | EventType::MESSAGE_DELETE
                 | EventType::MESSAGE_DELETE_BULK
                 | EventType::MESSAGE_UPDATE,
        )
        .build();
    let cache = InMemoryCache::from(cache_config);

    let mut events = cluster.events().await;
    while let Some(event) = events.next().await {
        cache.update(&event.1).await.expect("Failed to update cache");
        tokio::spawn(handle_event(event, http.clone()));
    }

    Ok(())
}

async fn handle_event(event: (u64, Event), _http: HttpClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        (_, Event::MessageCreate(_)) => {

        },
        (id, Event::ShardConnected(_)) => {
            println!("Connected on shard {}", id);
        }
        _ => {}
    }

    Ok(())
}
