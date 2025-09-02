#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("No credentials found, please authenticate with `slot auth login`")]
    Unauthorized,

    #[error("Malformed credentials, please reauthenticate with `slot auth login`")]
    MalformedCredentials,

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
