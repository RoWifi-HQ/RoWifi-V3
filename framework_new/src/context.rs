use twilight_http::Client as Http;

pub struct BotContext;

#[derive(Clone)]
pub struct CommandContext {
    pub http: Http
}