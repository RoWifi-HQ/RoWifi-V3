use crate::{
    arguments::FromArgs, command::ServiceRequest, context::CommandContext, error::RoError,
    CommandResult,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::EmbedFieldBuilder;

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

pub struct CommandHandler<F, T, R>
where
    F: Handler<T, R>,
    R: Future<Output = CommandResult> + Send,
{
    hnd: F,
    _t: PhantomData<(T, R)>,
}

impl<F, T, R> CommandHandler<F, T, R>
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

#[allow(clippy::type_complexity)]
impl<F, R, K> Service<(CommandContext, ServiceRequest)>
    for CommandHandler<F, (CommandContext, K), R>
where
    F: Handler<(CommandContext, K), R>,
    R: Future<Output = CommandResult> + Send + 'static,
    K: FromArgs,
{
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: (CommandContext, ServiceRequest)) -> Self::Future {
        match req.1 {
            ServiceRequest::Message(mut args) => match FromArgs::from_args(&mut args) {
                Ok(args) => {
                    let fut = self.hnd.call((req.0, args));
                    Box::pin(fut)
                }
                Err(err) => {
                    let fut = async move { Err(err.into()) };
                    Box::pin(fut)
                }
            },
            ServiceRequest::Interaction(options) => match FromArgs::from_interaction(&options) {
                Ok(args) => {
                    let fut = self.hnd.call((req.0, args));
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
                    embed = embed.field(EmbedFieldBuilder::new("Fields", fields_help).unwrap())
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
                        .embed(embed)
                        .unwrap()
                        .await?;
                    Ok(())
                };
                Box::pin(fut)
            }
        }
    }
}
