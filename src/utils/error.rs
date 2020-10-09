use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoError {
    #[error(transparent)]
    Database(#[from] mongodb::error::Error),

    #[error(transparent)]
    Serialization(#[from] bson::ser::Error),

    #[error(transparent)]
    Deserialization(#[from] bson::de::Error),

    #[error(transparent)]
    Roblox(#[from] reqwest::Error),

    #[error(transparent)]
    Discord(#[from] twilight_http::Error),

    #[error(transparent)]
    Command(#[from] CommandError)
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("{0}")]
    NicknameTooLong(String),

    #[error("You were found on the server blacklist. Reason: {0}")]
    Blacklist(String),

    #[error("This server has not been setup. Please ask the server owner to set it up")]
    NoRoGuild,

    #[error("Error in parsing the argument")]
    ParseArgument(String, String, String),

    #[error("Timeout reached. Please try again")]
    Timeout
}