use futures::FutureExt;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::EmbedBuilder;
use twilight_model::applications::interaction::CommandDataOption;

use crate::{
    arguments::{ArgumentError, Arguments, FromArgs},
    context::CommandContext,
    error::{CommandError, RoError},
    handler::{CommandHandler, Handler},
    prelude::{Color, EmbedExtensions},
    utils::RoLevel,
    CommandResult,
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
    pub master_name: String,
    pub names: &'static [&'static str],
    pub(crate) service: BoxedService,
    pub sub_commands: Vec<Command>,
    pub options: CommandOptions,
}

impl Command {
    pub fn builder() -> CommandBuilder {
        CommandBuilder::default()
    }

    fn _master_name(&mut self, top_name: &str) {
        self.master_name = format!("{} {}", top_name, self.names[0]).trim().to_string();
        for sub_cmd in self.sub_commands.iter_mut() {
            sub_cmd._master_name(&self.master_name);
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

    fn call(&mut self, mut req: (CommandContext, ServiceRequest)) -> Self::Future {
        let name = self.names[0];
        let ctx = req.0.clone();
        let master_name = self.master_name.clone();

        let fut = match req.1 {
            ServiceRequest::Message(ref mut args) => {
                if let Some(lit) = args.next() {
                    if let Some(sub_cmd) = self
                        .sub_commands
                        .iter_mut()
                        .find(|c| c.names.contains(&lit))
                    {
                        return sub_cmd.call(req);
                    }
                }
                args.back();
                self.service.call(req)
            }
            ServiceRequest::Interaction(ref top_options) => {
                for option in top_options {
                    if let CommandDataOption::SubCommand { name, options } = option {
                        if let Some(sub_cmd) = self
                            .sub_commands
                            .iter_mut()
                            .find(|c| c.names.contains(&name.as_str()))
                        {
                            req.1 = ServiceRequest::Interaction(options.clone());
                            return sub_cmd.call(req);
                        }
                    }
                }
                self.service.call(req)
            }
        };

        let fut = fut.then(move |res: Result<(), RoError>| async move {
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
                    handle_error(&err, ctx, &master_name).await;
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

async fn handle_error(err: &RoError, ctx: CommandContext, master_name: &str) {
    match err {
        RoError::Argument(arg_err) => match arg_err {
            ArgumentError::MissingArgument { usage, name } => {
                let content = format!(
                    "```{} {}\n\nExpected the {} argument\n\nFields Help:\n{}```",
                    master_name, usage.0, name, usage.1
                );
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .content(content)
                    .unwrap()
                    .await;
            }
            ArgumentError::ParseError {
                expected,
                usage,
                name,
            } => {
                let content = format!(
                    "```{} {}\n\nExpected {} to be {}\n\nFields Help:\n{}```",
                    master_name, usage.0, name, expected, usage.1
                );
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .content(content)
                    .unwrap()
                    .await;
            }
            ArgumentError::BadArgument => {
                //This shouldn't be happening but still report it to the user
            }
        },
        RoError::Command(cmd_err) => match cmd_err {
            CommandError::Blacklist(ref b) => { /*Handled invidually by the methods that raise this */
            }
            CommandError::Miscellanous(ref b) => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Command Failure")
                    .unwrap()
                    .color(Color::Red as u32)
                    .unwrap()
                    .description(b)
                    .unwrap()
                    .build()
                    .unwrap();
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .embed(embed)
                    .unwrap()
                    .await;
            }
            CommandError::Timeout => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Command Failure")
                    .unwrap()
                    .color(Color::Red as u32)
                    .unwrap()
                    .description("Timeout reached. Please try again")
                    .unwrap()
                    .build()
                    .unwrap();
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .embed(embed)
                    .unwrap()
                    .await;
            }
            CommandError::Ratelimit(ref d) => {
                let embed = EmbedBuilder::new()
                    .default_data()
                    .title("Command Failure")
                    .unwrap()
                    .color(Color::Red as u32)
                    .unwrap()
                    .description(format!(
                        "Ratelimit reached. You may retry this command in {} seconds",
                        d
                    ))
                    .unwrap()
                    .build()
                    .unwrap();
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .embed(embed)
                    .unwrap()
                    .await;
            }
            CommandError::NoRoGuild => {
                let embed = EmbedBuilder::new()
                    .default_data().title("Command Failure").unwrap().color(Color::Red as u32).unwrap()
                    .description("This server has not been set up. Please ask the server owner to do so using `!setup`").unwrap().build().unwrap();
                let _ = ctx
                    .bot
                    .http
                    .create_message(ctx.channel_id)
                    .embed(embed)
                    .unwrap()
                    .await;
            }
        },

        _ => {
            tracing::error!(err = ?err);
            let _ = ctx.bot.http.create_message(ctx.channel_id).content("There was an issue in executing. Please try again. If the issue persists, please contact our support server").unwrap().await;
            let content = format!(
                "Guild Id: {:?}\n Cluster Id: {}\nError: {:?}",
                ctx.guild_id, ctx.bot.cluster_id, err
            );
            ctx.log_error(&content).await;
        }
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
        let mut cmd = Command {
            options: self.options,
            names: self.names,
            service,
            sub_commands: self.sub_commands,
            master_name: "".into(),
        };
        cmd._master_name("");
        cmd
    }

    pub fn handler<F, R, K>(self, handler: F) -> Command
    where
        F: Handler<(CommandContext, K), R> + Send + 'static,
        R: Future<Output = CommandResult> + Send + 'static,
        K: FromArgs + Send + 'static,
    {
        let mut cmd = Command {
            options: self.options,
            names: self.names,
            service: Box::new(CommandHandler::new(handler)),
            sub_commands: self.sub_commands,
            master_name: "".into(),
        };
        cmd._master_name("");
        cmd
    }
}
