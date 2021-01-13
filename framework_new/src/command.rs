use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use futures::FutureExt;
use tower::Service;
use twilight_model::applications::CommandDataOption;

use crate::{Arguments, CommandContext, CommandHandler, CommandResult, FromArgs, Handler, RoError, context::BotContext, utils::RoLevel, error::CommandError};

pub type BoxedService = Box<
    dyn Service<
        (CommandContext, ServiceRequest),
        Response = (),
        Error = RoError,
        Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>,
    > + Send,
>;

pub enum ServiceRequest {
    Message(Arguments),
    Interaction(Vec<CommandDataOption>),
}

#[derive(Default)]
pub struct CommandOptions {
    pub level: RoLevel,
    pub bucket: Option<&'static str>,
    pub desc: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub hidden: bool,
    pub group: Option<&'static str>,
}

pub struct Command {
    pub names: &'static [&'static str],
    pub(crate) service: BoxedService,
    pub sub_commands: HashMap<String, Command>,
    pub options: CommandOptions,
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
            service: Box::new(CommandHandler::new(handler)),
            sub_commands: HashMap::new(),
            options: CommandOptions::default(),
        }
    }
}

impl Service<(CommandContext, ServiceRequest)> for Command {
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: (CommandContext, ServiceRequest)) -> Self::Future {
        let name = self.names[0];
        let ctx = req.0.clone();
        let fut = self.service.call(req).then(move |res: Result<(), RoError>| async move {
            match res {
                Ok(r) => {
                    if let Ok(metric) = ctx.bot.stats.command_counts.get_metric_with_label_values(&[&name]) {
                        metric.inc();
                    }
                    Ok(r)
                },
                Err(err) => {
                    handle_error(&err, ctx).await;
                    Err(err)
                }
            }
        });
        Box::pin(fut)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("Command")
            .field("name", &self.names)
            .finish()
    }
}

async fn handle_error(err: &RoError, bot: CommandContext) {
    match err {
        RoError::Argument(arg_err) => {},
        RoError::Command(cmd_err) => match cmd_err {
            CommandError::Blacklist(ref b) => {},
            CommandError::Miscellanous(ref b) => {}
            CommandError::Timeout => {}
        },
        _ => {}
    }
}
