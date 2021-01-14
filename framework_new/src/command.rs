use futures::FutureExt;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;
use twilight_model::applications::CommandDataOption;

use crate::{
    context::BotContext, error::CommandError, utils::RoLevel, Arguments, CommandContext,
    CommandHandler, CommandResult, FromArgs, Handler, RoError,
};

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
    pub desc: Option<&'static str>,
    pub examples: &'static [&'static str],
    pub hidden: bool,
    pub group: Option<&'static str>,
}

pub struct Command {
    pub names: &'static [&'static str],
    pub(crate) service: BoxedService,
    pub sub_commands: Vec<Command>,
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
            sub_commands: Vec::new(),
            options: CommandOptions::default(),
        }
    }

    pub fn builder() -> CommandBuilder {
        CommandBuilder::default()
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
        let fut = self
            .service
            .call(req)
            .then(move |res: Result<(), RoError>| async move {
                match res {
                    Ok(r) => {
                        if let Ok(metric) = ctx
                            .bot
                            .stats
                            .command_counts
                            .get_metric_with_label_values(&[&name])
                        {
                            metric.inc();
                        }
                        Ok(r)
                    }
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
        RoError::Argument(arg_err) => {}
        RoError::Command(cmd_err) => match cmd_err {
            CommandError::Blacklist(ref b) => {}
            CommandError::Miscellanous(ref b) => {}
            CommandError::Timeout => {}
        },
        _ => {}
    }
}

#[derive(Default)]
pub struct CommandBuilder {
    options: CommandOptions,
    names: &'static [&'static str],
    sub_commands: Vec<Command>,
}

impl CommandBuilder {
    pub fn level(mut self, level: RoLevel) -> Self {
        self.options.level = level;
        self
    }

    pub fn description(mut self, desc: &'static str) -> Self {
        self.options.desc = Some(desc);
        self
    }

    pub fn examples(mut self, examples: &'static [&'static str]) -> Self {
        self.options.examples = examples;
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.options.hidden = hidden;
        self
    }

    pub fn group(mut self, group: &'static str) -> Self {
        self.options.group = Some(group);
        self
    }

    pub fn names(mut self, names: &'static [&'static str]) -> Self {
        self.names = names;
        self
    }

    pub fn sub_command(mut self, sub_command: Command) -> Self {
        self.sub_commands.push(sub_command);
        self
    }

    pub fn service(self, service: BoxedService) -> Command {
        Command {
            options: self.options,
            names: self.names,
            service,
            sub_commands: self.sub_commands,
        }
    }

    pub fn handler<F, R, K>(self, handler: F) -> Command
    where
        F: Handler<(CommandContext, K), R> + Send + 'static,
        R: Future<Output = CommandResult> + Send + 'static,
        K: FromArgs + Send + 'static,
    {
        Command {
            options: self.options,
            names: self.names,
            service: Box::new(CommandHandler::new(handler)),
            sub_commands: self.sub_commands,
        }
    }
}
