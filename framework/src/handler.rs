use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::EmbedFieldBuilder;

use crate::{
    arguments::FromArgs, context::CommandContext, error::RoError, CommandResult, ServiceRequest,
};

pub trait Handler<T, R>: Clone + 'static
where
    R: Future<Output = CommandResult>,
{
    fn call(&self, ctx: CommandContext, param: T) -> R;
}

pub struct CommandHandler<F, T, R>
where
    F: Handler<T, R>,
    T: FromArgs,
    R: Future<Output = CommandResult>,
{
    hnd: F,
    _p: PhantomData<(T, R)>,
}

impl<F, T, R> CommandHandler<F, T, R>
where
    F: Handler<T, R>,
    T: FromArgs,
    R: Future<Output = CommandResult>,
{
    pub fn new(hnd: F) -> Self {
        Self {
            hnd,
            _p: PhantomData,
        }
    }
}

impl<F, T, R> Clone for CommandHandler<F, T, R>
where
    F: Handler<T, R>,
    T: FromArgs,
    R: Future<Output = CommandResult>,
{
    fn clone(&self) -> Self {
        Self {
            hnd: self.hnd.clone(),
            _p: PhantomData,
        }
    }
}

#[allow(clippy::type_complexity)]
impl<F, R, K> Service<(CommandContext, ServiceRequest)> for CommandHandler<F, K, R>
where
    F: Handler<K, R>,
    R: Future<Output = CommandResult> + Send + 'static,
    K: FromArgs,
{
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: (CommandContext, ServiceRequest)) -> Self::Future {
        match req.1 {
            ServiceRequest::Message(mut args) => match K::from_args(&mut args) {
                Ok(args) => {
                    let fut = self.hnd.call(req.0, args);
                    Box::pin(fut)
                }
                Err(err) => {
                    let fut = async move { Err(err.into()) };
                    Box::pin(fut)
                }
            },
            ServiceRequest::Interaction(options) => match K::from_interaction(&options) {
                Ok(args) => {
                    let fut = self.hnd.call(req.0, args);
                    Box::pin(fut)
                }
                Err(err) => {
                    let fut = async move { Err(err.into()) };
                    Box::pin(fut)
                }
            },
            ServiceRequest::Help(_args, mut embed) => {
                let (usage, fields_help) = K::generate_help();
                if !fields_help.is_empty() {
                    embed = embed.field(EmbedFieldBuilder::new("Fields", fields_help));
                }
                let mut embed = embed.build().unwrap();
                if let Some(field) = embed.fields.iter_mut().find(|f| f.name.eq("Usage")) {
                    field.value = format!("`{} {}`", field.value, usage);
                }

                let ctx = req.0;
                let fut = async move {
                    ctx.bot
                        .http
                        .create_message(ctx.channel_id)
                        .embeds(&[embed])
                        .unwrap()
                        .exec()
                        .await?;
                    Ok(())
                };
                Box::pin(fut)
            }
        }
    }
}

impl<F, R> Handler<(), R> for F
where
    F: Fn(CommandContext) -> R + Clone + 'static,
    R: Future<Output = CommandResult>,
{
    fn call(&self, ctx: CommandContext, (): ()) -> R {
        (self)(ctx)
    }
}

impl<F, K, R> Handler<(K,), R> for F
where
    F: Fn(CommandContext, K) -> R + Clone + 'static,
    R: Future<Output = CommandResult>,
{
    fn call(&self, ctx: CommandContext, (param,): (K,)) -> R {
        (self)(ctx, param)
    }
}
