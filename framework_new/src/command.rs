use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::{CommandContext, CommandResult, FromArgs, Handler, HandlerService, RoError, Service};

pub type BoxedService = Box<
    dyn Service<
        (CommandContext, String),
        Response = (),
        Error = RoError,
        Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>,
    > + Send,
>;

#[derive(Default)]
pub struct CommandOptions {
    pub level: RoLevel,
    pub bucket: Option<&'static str>,
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub hidden: bool,
    pub group: Option<&'static str>
}

pub struct Command {
    pub names: &'static [&'static str],
    pub(crate) service: BoxedService,
    pub sub_commands: Arc<HashMap<String, Command>>,
    pub options: CommandOptions,
    pub(crate) before: Option<BoxedService>,
}

impl Command {
    pub fn new<F, R, K>(names: &'static [&'static str], handler: F) -> Self
    where
        F: Handler<(CommandContext, K), R> + Send + 'static,
        R: Future<Output = CommandResult> + Send + 'static,
        K: FromArgs + Send + 'static,
    {
        Self {
            names,
            service: Box::new(HandlerService::new(handler)),
            sub_commands: Arc::new(HashMap::new()),
            options: CommandOptions::default(),
            before: None,
        }
    }
}

unsafe impl Sync for Command {}

impl Service<(CommandContext, String)> for Command {
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: (CommandContext, String)) -> Self::Future {
        Box::pin(self.service.call(req))
    }
}

#[derive(Debug, PartialEq, Ord, PartialOrd, Eq)]
#[repr(i8)]
pub enum RoLevel {
    Creator = 3,
    Admin = 2,
    Trainer = 1,
    Normal = 0,
}

impl Default for RoLevel {
    fn default() -> Self {
        RoLevel::Normal
    }
}
