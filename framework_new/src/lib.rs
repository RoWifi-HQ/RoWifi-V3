#![allow(dead_code)]
#![allow(unused_variables)]

pub mod command;
pub mod context;
pub mod error;
pub mod handler;
pub mod service;
pub mod prelude;

use std::{future::{Future, ready}, pin::Pin, task::{Context, Poll}};
use twilight_model::{gateway::event::Event, id::{UserId, ChannelId}};
use twilight_http::Client as Http;
use twilight_command_parser::Arguments;

use command::Command;
use context::CommandContext;
use handler::{Handler, HandlerService};
use error::RoError;
use service::Service;

pub type CommandResult = Result<(), RoError>;

pub trait FromArgs {
    fn from_args(args: &mut Arguments<'_>) -> Self where Self: Sized;
}

pub struct Framework {
    ctx: CommandContext,
    cmds: Vec<Command>
}

impl Framework {
    pub fn new(http: Http) -> Self
    {
        Self {
            ctx: CommandContext {
                http
            },
            cmds: vec![Command::new(update)]
        }
    }
}

impl Service<&Event> for Framework {
    type Response = ();
    type Error = RoError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: &Event) -> Self::Future {
        match req {
            Event::MessageCreate(msg) => {
                if let Some(cmd_str) = msg.content.split_ascii_whitespace().next() {
                    
                }
            },
            _ => {}
        }
        let fut = ready(Ok(()));
        Box::pin(fut)
    }
}

pub struct UpdateArguments {
    pub user_id: UserId
}

impl FromArgs for UpdateArguments {
    fn from_args(args: &mut Arguments<'_>) -> Self {
        UpdateArguments {user_id: UserId(1)}
    }
}

async fn update(ctx: CommandContext, args: UpdateArguments) -> CommandResult {
    let _res = ctx.http.create_message(ChannelId(460129585846288388)).content("Test").unwrap().await;
    Ok(())
}