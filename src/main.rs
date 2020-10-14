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
use tokio::stream::StreamExt;
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;
use twilight_standby::Standby;

use cache::Cache;
use commands::*;
use framework::{context::Context, Framework};
use models::{configuration::Configuration, stats::BotStats};
use services::*;
use utils::{Database, Logger, Patreon, Roblox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    //tracing_log::LogTracer::init()?;

    let token = env::var("DISC_TOKEN").expect("Expected Discord Token in the enviornment");
    let conn_string = env::var("DB_CONN").expect("Expceted database connection in env");
    let premium_features = env::var("PREMIUM_FEATURES")?
        .as_str()
        .parse::<bool>()
        .expect("Expected premium toggle");
    let patreon_key = env::var("PATREON").expect("Expected a Patreon key in the environment");
    let cluster_id = env::var("CLUSTER_ID")
        .expect("Expected the cluster id in the enviornment")
        .parse::<u64>()
        .unwrap();
    let total_shards = env::var("TOTAL_SHARDS")
        .expect("Expected the number of shards in the environment")
        .parse::<u64>()
        .unwrap();

    let scheme = ShardScheme::Auto;
    let http = HttpClient::new(&token);
    let app_info = http.current_user().await?;
    let owners = DashSet::new();
    let owner = http.current_user_application().await?.owner.id;
    owners.insert(owner);

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

    let cache = Cache::new();
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
            .owners(owners),
    );
    let patreon = Patreon::new(&patreon_key);
    let stats = Arc::new(BotStats::new(cluster_id));

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });
    tokio::spawn(run_metrics_server(stats.clone()));

    let context = Context::new(
        0, http, cache, database, roblox, standby, cluster, logger, config, patreon, stats,
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
        .command(&SERVERINFO_COMMAND)
        .command(&BOTINFO_COMMAND)
        .command(&USERINFO_COMMAND)
        .command(&SUPPORT_COMMAND)
        .help(&HELP_COMMAND)
        .bucket("update-multiple", Duration::from_secs(12 * 3600), 3);

    let framework = Arc::new(Box::new(framework));
    let event_handler = EventHandler::default();

    if premium_features {
        let context_ad = context.clone();
        tokio::spawn(async move {
            let _ = auto_detection(context_ad, total_shards).await;
        });
    }

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
            e.handle_event(event.0, &event.1, &c).await.unwrap();
            f.handle_event(&event.1, &c).await;
            c.stats.update(&event.1);
        });
    }

    Ok(())
}

async fn run_metrics_server(stats: Arc<BotStats>) {
    let addr = ([127, 0, 0, 1], 9898).into();
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
