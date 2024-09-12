use account_sdk::signers::SignError;
use starknet::core::utils::NonAsciiNameError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("No credentials found, please authenticate with `slot auth login`")]
    Unauthorized,

    #[error("Legacy credentials found, please reauthenticate with `slot auth login`")]
    LegacyCredentials,

    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("Invalid OAuth token, please authenticate with `slot auth login`")]
    InvalidOAuth,

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("Invalid method name: {0}")]
    InvalidMethodName(NonAsciiNameError),

    #[error(transparent)]
    Signing(#[from] SignError),
}
