use futures::{
    future::{ready, Either, Ready},
    Future, FutureExt,
};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tower::{Layer, Service};
use transient_dashmap::TransientDashMap;
use twilight_model::id::GuildId;

use crate::{
    command::ServiceRequest,
    context::CommandContext,
    error::{CommandError, RoError},
};

pub struct BucketLayer {
    pub time: Duration,
    pub calls: u64,
}

impl<S> Layer<S> for BucketLayer {
    type Service = BucketService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        BucketService {
            time: self.time,
            guilds: Arc::new(TransientDashMap::new(self.time)),
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
                if g.object == 0 {
                    return g.expiration.checked_duration_since(Instant::now());
                }
                None
            }
            None => None,
        }
    }
}

impl<S> Service<(CommandContext, ServiceRequest)> for BucketService<S>
where
    S: Service<(CommandContext, ServiceRequest), Error = RoError> + 'static,
{
    type Response = S::Response;
    type Error = RoError;
    type Future = Either<
        Ready<Result<Self::Response, Self::Error>>,
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: (CommandContext, ServiceRequest)) -> Self::Future {
        if let Some(guild_id) = req.0.guild_id {
            if let Some(duration) = self.guilds.get(&guild_id) {
                return Either::Left(ready(Err(RoError::Command(CommandError::Ratelimit(
                    duration.object,
                )))));
            }

            let guilds = self.guilds.clone();
            let calls = self.calls;
            let fut = self.service.call(req).then(move |res| async move {
                if res.is_ok() {
                    take(guilds, calls, guild_id);
                }

                res
            });

            return Either::Right(Box::pin(fut));
        }

        Either::Right(Box::pin(self.service.call(req)))
    }
}

fn take(
    guilds: Arc<TransientDashMap<GuildId, u64>>,
    calls: u64,
    guild_id: GuildId,
) -> Option<Duration> {
    let (new_remaining, expiration) = match guilds.get(&guild_id) {
        Some(g) => {
            let remaining = g.object;
            if remaining == 0 {
                return g.expiration.checked_duration_since(Instant::now());
            }
            (remaining - 1, g.expiration)
        }
        None => {
            guilds.insert(guild_id, calls - 1);
            return None;
        }
    };
    guilds.insert_with_expiration(guild_id, new_remaining, expiration);
    None
}
