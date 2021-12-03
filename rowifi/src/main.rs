#![deny(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::needless_lifetimes,
    clippy::let_underscore_drop,
    clippy::non_ascii_literal
)]

mod commands;
mod services;
mod utils;

use chacha20poly1305::{aead::NewAead, ChaCha20Poly1305, Key};
use commands::{rankbinds_config, user_config, custombinds_config, groupbinds_config, assetbinds_config, group_config};
use deadpool_redis::{Manager as RedisManager, Pool as RedisPool, Runtime};
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use patreon::Client as PatreonClient;
use prometheus::{Encoder, TextEncoder};
use roblox::Client as RobloxClient;
use rowifi_cache::Cache;
use rowifi_database::Database;
use rowifi_framework::{context::BotContext, Framework};
use rowifi_models::{
    discord::{gateway::Intents, guild::Permissions},
    stats::BotStats,
};
use services::EventHandler;
use std::{
    collections::HashMap,
    env,
    error::Error,
    future::{ready, Ready},
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{task::JoinError, time::sleep};
use tokio_stream::StreamExt;
use tower::Service;
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event,
};
use twilight_http::Client as HttpClient;
use twilight_standby::Standby;

pub struct RoWifi {
    pub framework: Framework,
    pub event_handler: EventHandler,
    pub bot: BotContext,
}

impl Service<(u64, Event)> for RoWifi {
    type Response = ();
    type Error = JoinError;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, event: (u64, Event)) -> Self::Future {
        self.bot
            .cache
            .update(&event.1)
            .expect("Failed to update cache");
        self.bot.standby.process(&event.1);
        self.bot.stats.update(&event.1);
        let fut = self.framework.call(&event.1);
        let eh_fut = self.event_handler.call((event.0, event.1));
        tokio::spawn(async move {
            if let Err(err) = eh_fut.await {
                tracing::error!(err = ?err);
            }
            let _ = fut.await;
        });
        ready(Ok(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let token = env::var("DISC_TOKEN").expect("Expected Discord Token in the enviornment");
    let connection_string = env::var("CONN_STRING").unwrap();
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
    let shards_per_cluster = env::var("SHARDS_PER_CLUSTER")
        .expect("Expected shards per cluster in the environment")
        .parse::<u64>()
        .unwrap();
    let pod_ip = env::var("POD_IP").expect("Expected the pod ip in the environment");
    let cipher_key = env::var("CIPHER_KEY").expect("Expected the cipher key in the environment");
    let redis_conn =
        env::var("REDIS_CONN").expect("Expected the redis connection in the environment");
    let proxy = env::var("PROXY").ok();
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
    let mut config = HttpClient::builder().token(token.clone());
    if let Some(proxy) = proxy {
        config = config.proxy(proxy, true).ratelimiter(None);
    }
    let http = Arc::new(config.build());
    let app_info = http.current_user().exec().await?.model().await?;

    let mut owners = Vec::new();
    let current_user = http
        .current_user_application()
        .exec()
        .await?
        .model()
        .await?;
    let owner = current_user.owner.id;
    http.set_application_id(current_user.id);
    owners.push(owner);

    let (cluster, mut events) = Cluster::builder(
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
    let cluster = Arc::new(cluster);

    let stats = Arc::new(BotStats::new(cluster_id));
    let cache = Cache::new(stats.clone());
    let standby = Standby::new();

    let redis = RedisPool::builder(RedisManager::new(redis_conn).unwrap())
        .max_size(4)
        .runtime(Runtime::Tokio1)
        .recycle_timeout(Some(Duration::from_secs(30)))
        .wait_timeout(Some(Duration::from_secs(30)))
        .create_timeout(Some(Duration::from_secs(30)))
        .build()
        .unwrap();
    let _res = redis.get().await.expect("Redis Connection failed");

    let database = Database::new(&connection_string).await;
    let roblox = RobloxClient::new(redis.clone());
    let patreon = PatreonClient::new(&patreon_key);

    let cipher_key = Key::from_slice(cipher_key.as_bytes());
    let cipher = ChaCha20Poly1305::new(cipher_key);

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
        total_shards,
        shards_per_cluster,
        cipher,
    );
    let framework = Framework::new(
        bot.clone(),
        Permissions::SEND_MESSAGES
            | Permissions::EMBED_LINKS
            | Permissions::MANAGE_ROLES
            | Permissions::MANAGE_NICKNAMES,
    )
    .configure(user_config)
    .configure(rankbinds_config)
    // .configure(analytics_config)
    .configure(assetbinds_config)
    // .configure(backup_config)
    // .configure(blacklists_config)
    .configure(custombinds_config)
    // .configure(events_config)
    .configure(group_config)
    // .configure(api_config)
    .configure(groupbinds_config)
    // .configure(settings_config)
    // .configure(premium_config);
    ;

    let event_handler = EventHandler::new(&bot);
    let mut rowifi = RoWifi {
        framework,
        event_handler,
        bot,
    };

    while let Some(event) = events.next().await {
        let _ = rowifi.call(event).await;
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
