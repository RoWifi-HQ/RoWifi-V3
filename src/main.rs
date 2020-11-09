mod cache;
mod commands;
mod framework;
mod models;
mod rolang;
mod services;
mod utils;

use dashmap::DashSet;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use prometheus::{Encoder, TextEncoder};
use std::{env, error::Error, sync::Arc, time::Duration};
use tokio::{stream::StreamExt, time::delay_for};
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_http::Client as HttpClient;
use twilight_model::{gateway::Intents, id::UserId};
use twilight_standby::Standby;

use cache::Cache;
use commands::*;
use framework::{context::Context, Framework};
use models::{
    configuration::{BotConfig, Configuration},
    stats::BotStats,
};
use services::*;
use utils::{Database, Logger, Patreon, Roblox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    //tracing_log::LogTracer::init()?;

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
    delay_for(Duration::from_secs(cluster_id * 60)).await;

    let scheme = ShardScheme::Range {
        from: cluster_id * shards_per_cluster,
        to: cluster_id * shards_per_cluster + shards_per_cluster - 1,
        total: total_shards,
    };
    let http = HttpClient::new(&token);
    let app_info = http.current_user().await?;

    let owners = DashSet::new();
    let owner = http.current_user_application().await?.owner.id;
    owners.insert(owner);

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
    let roblox = Roblox::new();
    let logger = Arc::new(Logger {
        debug_webhook: env::var("LOG_DEBUG")
            .expect("Expected the debug webhook in the environment"),
        main_webhook: env::var("LOG_MAIN").expect("Expected the main webhook in the environment"),
        premium_webhook: env::var("LOG_PREMIUM")
            .expect("Expected the premium webhook in the environment"),
    });
    let config = Arc::new(
        Configuration::default()
            .default_prefix("!")
            .on_mention(app_info.id)
            .owners(owners)
            .council(council),
    );
    let patreon = Patreon::new(&patreon_key);
    let bot_config = Arc::new(BotConfig {
        cluster_id,
        shards_per_cluster,
        total_shards,
    });

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });
    tokio::spawn(run_metrics_server(pod_ip, stats.clone()));

    let context = Context::new(
        http, cache, database, roblox, standby, cluster, logger, config, patreon, stats, bot_config,
    );
    let framework = Framework::default()
        .command(&UPDATE_COMMAND)
        .command(&VERIFY_COMMAND)
        .command(&REVERIFY_COMMAND)
        .command(&RANKBINDS_COMMAND)
        .command(&GROUPBINDS_COMMAND)
        .command(&CUSTOMBINDS_COMMAND)
        .command(&ASSETBINDS_COMMAND)
        .command(&BLACKLISTS_COMMAND)
        .command(&SETTINGS_COMMAND)
        .command(&SETUP_COMMAND)
        .command(&UPDATE_ALL_COMMAND)
        .command(&UPDATE_ROLE_COMMAND)
        .command(&BACKUP_COMMAND)
        .command(&PREMIUM_COMMAND)
        .command(&ANALYTICS_COMMAND)
        .command(&EVENTS_COMMAND)
        .command(&SERVERINFO_COMMAND)
        .command(&BOTINFO_COMMAND)
        .command(&USERINFO_COMMAND)
        .command(&SUPPORT_COMMAND)
        .help(&HELP_COMMAND)
        .bucket("update-multiple", Duration::from_secs(12 * 3600), 3);

    let framework = Arc::new(Box::new(framework));
    let event_handler = EventHandler::default();

    let mut events = context.cluster.events();
    while let Some(event) = events.next().await {
        let c = context.clone();
        let f = framework.clone();
        let e = event_handler.clone();
        tracing::trace!(event = ?event.1.kind());
        context
            .cache
            .update(&event.1)
            .expect("Failed to update cache");
        context.standby.process(&event.1);

        tokio::spawn(async move {
            if let Err(err) = e.handle_event(event.0, &event.1, &c).await {
                tracing::error!(err = ?err, "Error in event handler");
            }
            f.handle_event(&event.1, &c).await;
            c.stats.update(&event.1);
        });
    }

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
