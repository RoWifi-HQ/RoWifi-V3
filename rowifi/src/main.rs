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
    clippy::non_ascii_literal,
    clippy::collapsible_if,
    clippy::redundant_closure_for_method_calls
)]

mod commands;
mod services;
mod utils;

use axum::{
    routing::{get, post},
    Extension, Json, Router, Server,
};
use commands::{
    analytics_config, assetbinds_config, backup_config, blacklists_config, custombinds_config,
    events_config, group_config, groupbinds_config, premium_config, rankbinds_config,
    settings_config, user_config,
};
use deadpool_redis::{Manager as RedisManager, Pool as RedisPool, Runtime};
use patreon::Client as PatreonClient;
use prometheus::{Encoder, TextEncoder};
use roblox::Client as RobloxClient;
use rowifi_cache::Cache;
use rowifi_database::Database;
use rowifi_framework::{context::BotContext, Framework};
use rowifi_models::{
    discord::{gateway::Intents, guild::Permissions},
    id::{GuildId, UserId},
    stats::BotStats,
};
use serde::Deserialize;
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
    let primary_key = env::var("PRIMARY_KEY").expect("Expected the cipher key in the environment");
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
    let owner = UserId(current_user.owner.id);
    owners.push(owner);

    let (cluster, mut events) = Cluster::builder(
        token,
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
        .max_size(16)
        .runtime(Runtime::Tokio1)
        .recycle_timeout(Some(Duration::from_secs(30)))
        .wait_timeout(Some(Duration::from_secs(30)))
        .create_timeout(Some(Duration::from_secs(30)))
        .build()
        .unwrap();
    let _res = redis.get().await.expect("Redis Connection failed");

    let database = Database::new(&connection_string, &primary_key).await;
    let roblox = RobloxClient::new(redis.clone());
    let patreon = PatreonClient::new(&patreon_key);

    let cluster_spawn = cluster.clone();
    tokio::spawn(async move {
        cluster_spawn.up().await;
    });
    tokio::spawn(run_server(pod_ip, stats.clone(), cache.clone()));

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
        current_user.id,
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
    .configure(analytics_config)
    .configure(assetbinds_config)
    .configure(backup_config)
    .configure(blacklists_config)
    .configure(custombinds_config)
    .configure(events_config)
    .configure(group_config)
    // .configure(api_config)
    .configure(groupbinds_config)
    .configure(settings_config)
    .configure(premium_config);

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

async fn run_server(pod_ip: String, stats: Arc<BotStats>, cache: Cache) {
    let router = Router::new()
        .route("/metrics", get(metrics))
        .route("/guilds", post(guilds))
        .layer(Extension(stats))
        .layer(Extension(cache));

    Server::bind(&pod_ip.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

#[allow(clippy::unused_async)]
async fn metrics(stats: Extension<Arc<BotStats>>) -> Vec<u8> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = stats.registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    buffer
}

#[derive(Deserialize, Debug)]
struct GuildsQuery {
    #[serde(default)]
    pub guild_ids: Vec<GuildId>,
}

#[allow(clippy::unused_async)]
async fn guilds(query: Json<GuildsQuery>, cache: Extension<Cache>) -> Json<Vec<GuildId>> {
    let mut bot_in_guilds = Vec::new();
    for guild_id in &query.guild_ids {
        let guild = cache.guild(*guild_id);
        if guild.is_some() {
            bot_in_guilds.push(*guild_id);
        }
    }

    Json(bot_in_guilds)
}
