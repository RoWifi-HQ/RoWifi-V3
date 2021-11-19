use itertools::Itertools;
use rowifi_models::discord::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::application_command::CommandOptionValue,
    },
    channel::message::MessageFlags,
};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;
use twilight_embed_builder::{EmbedBuilder, EmbedFieldBuilder};

use crate::{
    arguments::{ArgumentError, FromArgs},
    context::CommandContext,
    error::{CommandError, ErrorKind, RoError},
    extensions::EmbedExtensions,
    handler::{CommandHandler, Handler},
    utils::{Color, RoLevel},
    ServiceRequest,
};

type BoxedService = Box<
    dyn Service<
            (CommandContext, ServiceRequest),
            Response = (),
            Error = RoError,
            Future = Pin<Box<dyn Future<Output = Result<(), RoError>> + Send>>,
        > + Send,
>;

pub type CommandResult = Result<(), RoError>;

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
        for sub_cmd in &mut self.sub_commands {
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
                        .find(|c| c.names.iter().any(|c| c.eq_ignore_ascii_case(lit)))
                    {
                        return sub_cmd.call(req);
                    }
                }
                args.back();
                self.service.call(req)
            }
            ServiceRequest::Interaction(ref top_options) => {
                for option in top_options {
                    if let CommandOptionValue::SubCommand(options) = &option.value {
                        if let Some(sub_cmd) = self
                            .sub_commands
                            .iter_mut()
                            .find(|c| c.names.contains(&option.name.as_str()))
                        {
                            req.1 = ServiceRequest::Interaction(options.clone());
                            return sub_cmd.call(req);
                        }
                    }
                }
                self.service.call(req)
            }
            ServiceRequest::Help(ref mut args, ref embed) => {
                if let Some(lit) = args.next() {
                    if let Some(sub_cmd) = self
                        .sub_commands
                        .iter_mut()
                        .find(|c| c.names.iter().any(|c| c.eq_ignore_ascii_case(lit)))
                    {
                        return sub_cmd.call(req);
                    }
                }
                args.back();

                let mut embed = embed.clone();
                embed = embed
                    .description(format!(
                        "{}: {}",
                        self.names[0],
                        self.options.desc.unwrap_or("None")
                    ))
                    .field(EmbedFieldBuilder::new("Usage", self.master_name.clone()));
                if self.names.len() > 1 {
                    let aliases = self.names[1..].iter().map(|a| format!("`{}`", a)).join(" ");
                    embed = embed.field(EmbedFieldBuilder::new("Aliases", aliases));
                }
                if !self.options.examples.is_empty() {
                    let examples = self
                        .options
                        .examples
                        .iter()
                        .map(|e| format!("`{}`", e))
                        .join("\n");
                    embed = embed.field(EmbedFieldBuilder::new("Examples", examples));
                }
                if !self.sub_commands.is_empty() {
                    let subs = self
                        .sub_commands
                        .iter()
                        .filter(|c| !c.options.hidden)
                        .map(|c| format!("`{}`", c.names[0]))
                        .join(", ");
                    embed = embed.field(EmbedFieldBuilder::new("Subcommands", subs));
                }
                return self
                    .service
                    .call((req.0, ServiceRequest::Help(args.clone(), embed)));
            }
        };

        if ctx.bot.disabled_channels.contains(&ctx.channel_id)
            && !self.names.contains(&"command-channel")
        {
            if let (Some(id), Some(token)) = (ctx.interaction_id, ctx.interaction_token) {
                let http = ctx.bot.http.clone();
                let fut = async move {
                    let _ = http
                        .interaction_callback(
                            id,
                            &token,
                            &InteractionResponse::ChannelMessageWithSource(CallbackData {
                                allowed_mentions: None,
                                tts: None,
                                embeds: Vec::new(),
                                content: Some("Commands are disabled in this channel".into()),
                                flags: Some(MessageFlags::EPHEMERAL),
                                components: None,
                            }),
                        )
                        .exec()
                        .await;
                    Ok(())
                };
                return Box::pin(fut);
            }
            return Box::pin(async move { Ok(()) });
        }

        let http = ctx.bot.http.clone();
        let (interaction_id, interaction_token) =
            (ctx.interaction_id, ctx.interaction_token.clone());
        let fut = async move {
            if let (Some(id), Some(token)) = (interaction_id, interaction_token) {
                let _ = http
                    .interaction_callback(
                        id,
                        &token,
                        &InteractionResponse::DeferredChannelMessageWithSource(CallbackData {
                            allowed_mentions: None,
                            tts: None,
                            embeds: Vec::new(),
                            content: None,
                            flags: None,
                            components: None,
                        }),
                    )
                    .exec()
                    .await;
            }
            let res = fut.await;
            match res {
                Ok(_) => {
                    if let Ok(metric) = ctx
                        .bot
                        .stats
                        .command_counts
                        .get_metric_with_label_values(&[name])
                    {
                        metric.inc();
                    }
                    Ok(())
                }
                Err(err) => {
                    handle_error(&err, ctx, &master_name).await;
                    Err(err)
                }
            }
        };
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
    let (kind, err) = err.parts();

    #[allow(clippy::single_match_else)]
    match kind {
        ErrorKind::Command => {
            if let Some(err) = err.as_ref().and_then(|e| e.downcast_ref::<CommandError>()) {
                match err {
                    CommandError::Argument(arg_err) => match arg_err {
                        ArgumentError::MissingArgument { usage, name } => {
                            let content = format!(
                                "```{} {}\n\nExpected the {} argument\n\nFields Help:\n{}```",
                                master_name, usage.0, name, usage.1
                            );
                            match ctx.respond().content(&content) {
                                Ok(r) => {
                                    let _ = r.exec().await;
                                }
                                Err(err) => tracing::error!("handle_error: {}", err),
                            }
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
                            match ctx.respond().content(&content) {
                                Ok(r) => {
                                    let _ = r.exec().await;
                                }
                                Err(err) => tracing::error!("handle_error: {}", err),
                            }
                        }
                        ArgumentError::BadArgument => {
                            //This shouldn't be happening but still report it to the user
                        }
                    },
                    CommandError::Cancelled => {
                        let embed = EmbedBuilder::new()
                            .default_data()
                            .title("Command Failure")
                            .color(Color::Red as u32)
                            .description("Command was cancelled.")
                            .build()
                            .unwrap();
                        let _ = ctx.respond().embeds(&[embed]).unwrap().exec().await;
                    }
                    CommandError::Message(_) => todo!(),
                    CommandError::Timeout => {
                        let embed = EmbedBuilder::new()
                            .default_data()
                            .title("Command Failure")
                            .color(Color::Red as u32)
                            .description("Command timed out. Please try again.")
                            .build()
                            .unwrap();
                        let _ = ctx.respond().embeds(&[embed]).unwrap().exec().await;
                    }
                    CommandError::Ratelimit(d) => {
                        let embed = EmbedBuilder::new()
                            .default_data()
                            .title("Command Failure")
                            .color(Color::Red as u32)
                            .description(format!(
                                "Ratelimit reached. You may retry this command in {} seconds",
                                d.as_secs()
                            ))
                            .build()
                            .unwrap();
                        let _ = ctx.respond().embeds(&[embed]).unwrap().exec().await;
                    }
                }
            }
        }
        _ => {
            tracing::error!(err = ?err);
            let _ = ctx.respond().content("There was an issue in executing. Please try again. If the issue persists, please contact our support server").unwrap().exec().await;
            let content = format!(
                "```Guild Id: {:?}\nCommand:{}\nCluster Id: {}\nError: {:?}```",
                ctx.guild_id, master_name, ctx.bot.cluster_id, err
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
        F: Handler<K, R> + Send + 'static,
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
