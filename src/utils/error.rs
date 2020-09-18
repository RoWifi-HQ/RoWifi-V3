use thiserror::Error;

#[derive(Debug, Error)]
pub enum RoError {
    #[error("There was an problem in connecting to the database")]
    Database(#[from] mongodb::error::Error),

    #[error("There was an error in serializing your data")]
    Serialization(#[from] bson::ser::Error),

    #[error("There was an error in deserializng your data")]
    Deserialization(#[from] bson::de::Error),

    #[error("There was some problem in connecting to the Roblox API")]
    Roblox(#[from] reqwest::Error),

    #[error("There was some error in interacting with Discord")]
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