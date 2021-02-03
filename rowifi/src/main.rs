#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::single_match_else,
    clippy::filter_map,
    clippy::too_many_lines,
    dead_code
)]

mod commands;
mod services;

use commands::{rankbinds_config, update_config};
use dashmap::DashSet;
use framework_new::{
    context::BotContext,
    prelude::{RoError, Service, ServiceExt},
    Framework as NewFramework,
};
use futures::{
    future::{Either, Ready},
    Future,
};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use patreon::Client as PatreonClient;
use prometheus::{Encoder, TextEncoder};
use roblox::Client as RobloxClient;
use rowifi_cache::Cache;
use rowifi_database::Database;
use rowifi_models::stats::BotStats;
use services::EventHandler;
use std::{
    collections::HashMap,
    env,
    error::Error,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    task::{JoinError, JoinHandle},
    time::sleep,
};
use tokio_stream::StreamExt;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::Intents, id::UserId};
use twilight_standby::Standby;

pub struct RoWifi {
    pub framework: NewFramework,
    pub event_handler: EventHandler,
    pub bot: BotContext,
}

impl Service<(u64, Event)> for RoWifi {
    type Response = Result<(), RoError>;
    type Error = JoinError;
    type Future = JoinHandle<
        <Either<
            Ready<Result<(), RoError>>,
            Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>,
        > as Future>::Output,
    >;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: (u64, Event)) -> Self::Future {
        self.bot
            .cache
            .update(&event.1)
            .expect("Failed to update cache");
        self.bot.standby.process(&event.1);
        let fut = self.framework.call(&event.1);

        let join = tokio::spawn(fut);

        join
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = env::var("DISC_TOKEN").expect("Expected Discord Token in the enviornment");
    let conn_string = env::var("DB_CONN").expect("Expceted database connection in env");
    let patreon_key = env::var("PATREON").expect("Expected a Patreon key in the environment");
    let cluster_id = env::var("CLUSTER_ID")
        .expect("Expected the cluster id in the enviornment")
        .split('-')
        .last()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let total_shards = env::var("TOTAL_SHARDS")
        .expect("Expected the number of shards in the environment")
        .parse::<u64>()
        .unwrap();
    let council_members = env::var("COUNCIL")
        .expect("Expected council members in the enviornment")
        .split('|')
        .map(|c| c.parse::<u64>().unwrap())
        .collect::<Vec<_>>();
    let shards_per_cluster = env::var("SHARDS_PER_CLUSTER")
        .expect("Expected shards per cluster in the environment")
        .parse::<u64>()
        .unwrap();
    let pod_ip = env::var("POD_IP").expect("Expected the pod ip in the environment");
    sleep(Duration::from_secs(cluster_id * 60)).await;

    let mut webhooks = HashMap::new();
    let debug_webhook =
        env::var("LOG_DEBUG").expect("Expected the debug webhook in the environment");
    let error_webhook =
        env::var("LOG_ERROR").expect("Expected the debug webhook in the environment");
    let premium_webhook =
        env::var("LOG_PREMIUM").expect("Expected the debug webhook in the environment");
    webhooks.insert("debug", debug_webhook.as_str());
    webhooks.insert("error", error_webhook.as_str());
    webhooks.insert("premium", premium_webhook.as_str());

    let scheme = ShardScheme::Range {
        from: cluster_id * shards_per_cluster,
        to: cluster_id * shards_per_cluster + shards_per_cluster - 1,
        total: total_shards,
    };
    let http = HttpClient::new(&token);
    let app_info = http.current_user().await?;

    let mut owners = Vec::new();
    let owner = http.current_user_application().await?.owner.id;
    owners.push(owner);

    let council = DashSet::new();
    for c in council_members {
        council.insert(UserId(c));
    }

    let cluster = Cluster::builder(
        &token,
        Intents::GUILD_MESSAGES
            | Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGE_REACTIONS,
    )
    .shard_scheme(scheme)
    .http_client(http.clone())
    .build()
    .await?;

    let stats = Arc::new(BotStats::new(cluster_id));
    let cache = Cache::new(stats.clone());
    let standby = Standby::new();

    let database = Database::new(&conn_string).await;
    let roblox = RobloxClient::default();
    let patreon = PatreonClient::new(&patreon_key);

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });
    tokio::spawn(run_metrics_server(pod_ip, stats.clone()));

    let bot = BotContext::new(
        app_info.id.to_string(),
        "!".into(),
        &owners,
        http,
        cache,
        cluster.clone(),
        standby,
        database,
        roblox,
        patreon,
        stats,
        webhooks,
        cluster_id,
    );
    let framework = NewFramework::new(bot.clone())
        .configure(update_config)
        .configure(rankbinds_config);

    let event_handler = EventHandler::default();
    let rowifi = RoWifi {
        framework,
        event_handler,
        bot,
    };
    let events = rowifi.bot.cluster.events();
    let mut event_responses = rowifi.call_all(events);
    while let Some(_res) = event_responses.next().await {}

    // let mut events = bot.cluster.events();
    // while let Some(event) = events.next().await {
    //     let b = bot.clone();
    //     let nfc = new_framework.clone();
    //     let _e = event_handler.clone();
    //     tracing::trace!(event = ?event.1.kind());
    //     b.cache.update(&event.1).expect("Failed to update cache");
    //     b.standby.process(&event.1);

    //     tokio::spawn(async move {
    //         // if let Err(err) = e.handle_event(event.0, &event.1, &c).await {
    //         //     tracing::error!(err = ?err, "Error in event handler");
    //         // }
    //         //f.handle_event(&event.1, &c).await;
    //         let fut = nfc.call(&event.1);
    //         if let Err(err) = fut.await {
    //             tracing::error!(err = ?err);
    //         }
    //         //c.stats.update(&event.1);
    //     });
    // }

    Ok(())
}

async fn run_metrics_server(pod_ip: String, stats: Arc<BotStats>) {
    let addr = format!("{}:{}", pod_ip, 9000).parse().unwrap();
    let metric_service = make_service_fn(move |_| {
        let stats = stats.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                let mut buffer = vec![];
                let encoder = TextEncoder::new();
                let metric_families = stats.registry.gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();

                async move { Ok::<_, hyper::Error>(Response::new(Body::from(buffer))) }
            }))
        }
    });

    let server = Server::bind(&addr).serve(metric_service);
    if let Err(err) = server.await {
        tracing::error!(error = ?err, "Error from the metrics server: ");
    }
}
