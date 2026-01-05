use thiserror::Error;

pub type Result<T> = std::result::Result<T, TurnkeyError>;

#[derive(Error, Debug)]
pub enum TurnkeyError {
    #[error("Turnkey SDK error: {0}")]
    Sdk(#[from] turnkey_client::TurnkeyClientError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("API error [{code}]: {message}")]
    Api { code: String, message: String },
}
