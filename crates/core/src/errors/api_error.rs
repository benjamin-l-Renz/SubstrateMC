use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Directory or file already exists")]
    AlreadyExists,

    #[error("Error using tokio channels")]
    ChannelError,
}
