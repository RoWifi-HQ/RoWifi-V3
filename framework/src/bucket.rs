use futures_util::future::{Future, FutureExt};
use rowifi_models::id::GuildId;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};
use transient_dashmap::TransientDashMap;

use crate::{
    context::CommandContext,
    error::{CommandError, ErrorKind, RoError},
    ServiceRequest,
};

#[derive(Clone)]
pub struct BucketLayer {
    pub time: Duration,
    pub calls: u64,
    guilds: Arc<TransientDashMap<GuildId, u64>>,
}

impl BucketLayer {
    pub fn new(time: Duration, calls: u64) -> Self {
        Self {
            time,
            calls,
            guilds: Arc::new(TransientDashMap::new(time)),
        }
    }
}

impl<S> Layer<S> for BucketLayer {
    type Service = BucketService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BucketService {
            time: self.time,
            guilds: self.guilds.clone(),
            calls: self.calls,
            service: inner,
        }
    }
}

#[derive(Clone)]
pub struct BucketService<S> {
    pub time: Duration,
    pub guilds: Arc<TransientDashMap<GuildId, u64>>,
    pub calls: u64,
    service: S,
}

impl<S> BucketService<S> {
    pub fn get(&self, guild_id: GuildId) -> Option<Duration> {
        match self.guilds.get(&guild_id) {
            Some(g) => {
                if **g == 0 {
                    return g.expiration().checked_duration_since(Instant::now());
                }
                None
            }
            None => None,
        }
    }
}

#[allow(clippy::type_complexity)]
impl<S> Service<(CommandContext, ServiceRequest)> for BucketService<S>
where
    S: Service<(CommandContext, ServiceRequest), Error = RoError> + 'static,
    S::Future: Send,
    S::Response: Send,
{
    type Response = S::Response;
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: (CommandContext, ServiceRequest)) -> Self::Future {
        if let Some(guild_id) = req.0.guild_id {
            if let Some(duration) = self.get(guild_id) {
                let fut = async move {
                    Err(RoError {
                        source: Some(Box::new(CommandError::Ratelimit(duration))),
                        kind: ErrorKind::Command,
                    })
                };
                return Box::pin(fut);
            }

            let guilds = self.guilds.clone();
            let calls = self.calls;
            let fut = self.service.call(req).then(move |res| async move {
                if res.is_ok() {
                    take(&guilds, calls, guild_id);
                }

                res
            });

            return Box::pin(fut);
        }

        Box::pin(self.service.call(req))
    }
}

fn take(
    guilds: &TransientDashMap<GuildId, u64>,
    calls: u64,
    guild_id: GuildId,
) -> Option<Duration> {
    let (new_remaining, expiration) = if let Some(g) = guilds.get(&guild_id) {
        let remaining = **g;
        if remaining == 0 {
            return g.expiration().checked_duration_since(Instant::now());
        }
        (remaining - 1, g.expiration())
    } else {
        guilds.insert(guild_id, calls - 1);
        return None;
    };
    guilds.insert_with_expiration(guild_id, new_remaining, expiration);
    None
}
