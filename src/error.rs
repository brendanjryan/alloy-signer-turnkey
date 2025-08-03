use thiserror::Error;

pub type Result<T> = std::result::Result<T, TurnkeyError>;

#[derive(Error, Debug)]
pub enum TurnkeyError {
    #[error("Turnkey SDK error: {0}")]
    Sdk(#[from] turnkey_client::TurnkeyClientError),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("API error [{code}]: {message}")]
    Api { code: String, message: String },

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("Signature parsing error: {0}")]
    SignatureParse(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Alloy signature error: {0}")]
    AlloySignature(#[from] alloy_primitives::SignatureError),

    #[error("Parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("String from UTF8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
