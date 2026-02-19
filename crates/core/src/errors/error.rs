use thiserror::Error;

#[derive(Debug, Error)]
pub enum SubstrateError {
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Resource already exists: {resource}")]
    AlreadyExists { resource: String },

    #[error("Failed to send message through channel")]
    ChannelSend,

    #[error("Failed to receive message from channel")]
    ChannelReceive,

    #[error("Upstream service returned an error response")]
    HttpStatus { body: String },

    #[error("HTTP request failed")]
    Request {
        #[from]
        source: reqwest::Error,
    },

    #[error("Filesystem operation failed")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON processing failed")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Failed to convert value: {details}")]
    ConversionError { details: String },

    #[error("Minecraft EULA has not been accepted")]
    Eula,

    #[error("Minecraft server failure: {message}")]
    McServerError { message: String },

    #[error("Failed to upload mod: {message}")]
    UploadModError { message: String },
}

#[cfg(feature = "actix-web")]
impl actix_web::error::ResponseError for SubstrateError {}
