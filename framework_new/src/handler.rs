use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use twilight_command_parser::Arguments;
use twilight_model::channel::Message;

use crate::{CommandContext, CommandResult, FromArgs, RoError, Service};

pub trait Handler<T, R>
where
    R: Future<Output = CommandResult>,
{
    fn call(&self, param: T) -> R;
}

impl<F, R, K> Handler<(CommandContext, K), R> for F
where
    F: Fn(CommandContext, K) -> R,
    R: Future<Output = CommandResult>,
    K: FromArgs,
{
    fn call(&self, param: (CommandContext, K)) -> R {
        (self)(param.0, param.1)
    }
}

pub struct HandlerService<F, T, R>
where
    F: Handler<T, R>,
    R: Future<Output = CommandResult> + Send,
{
    hnd: F,
    _t: PhantomData<(T, R)>,
}

impl<F, T, R> HandlerService<F, T, R>
where
    F: Handler<T, R>,
    R: Future<Output = CommandResult> + Send,
{
    pub fn new(handler: F) -> Self {
        Self {
            hnd: handler,
            _t: PhantomData,
        }
    }
}

impl<F, R, K> Service<(CommandContext, Message)> for HandlerService<F, (CommandContext, K), R>
where
    F: Handler<(CommandContext, K), R>,
    R: Future<Output = CommandResult> + Send + 'static,
    K: FromArgs,
{
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: (CommandContext, Message)) -> Self::Future {
        let mut arguments = Arguments::new(&req.1.content);
        let args = FromArgs::from_args(&mut arguments).ok().unwrap();
        let fut = self.hnd.call((req.0, args));
        Box::pin(fut)
    }
}
