use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, task::{Context, Poll}};
use twilight_model::channel::Message;

use crate::{Service, CommandContext, RoError, Handler, FromArgs, CommandResult, HandlerService};

pub type BoxedService = Box<dyn Service<
    (CommandContext, Message),
    Response = (),
    Error = RoError,
    Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>
> + Send>;

pub struct CommandOptions {}

pub struct Command {
    pub name: &'static str,
    pub(crate) service: BoxedService,
    pub sub_commands: Arc<HashMap<&'static str, Command>>
}

impl Command {
    pub fn new<F, R, K>(name: &'static str, handler: F) -> Self 
    where
        F: Handler<(CommandContext, K), R> + Send + 'static,
        R: Future<Output=CommandResult> + Send + 'static,
        K: FromArgs + Send + 'static
    {
        Self {
            name,
            service: Box::new(HandlerService::new(handler)),
            sub_commands: Arc::new(HashMap::new())
        }
    }
}

unsafe impl Sync for Command {}

impl Service<(CommandContext, Message)> for Command {
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: (CommandContext, Message)) -> Self::Future {
        Box::pin(self.service.call(req))
    }
}
