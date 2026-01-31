use std::fmt::Display;

use actix_web::ResponseError;

use crate::server::Servers;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
    HttpRequestError(reqwest::Error),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    ActixWsError(actix_web::Error),
    WebDataError(std::sync::PoisonError<Servers>),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "Not Found"),
            ApiError::InternalServerError => write!(f, "Internal Server Error"),
            ApiError::HttpRequestError(e) => write!(f, "HTTP Request Error: {}", e),
            ApiError::IoError(e) => write!(f, "IO Error: {}", e),
            ApiError::JsonError(e) => write!(f, "JSON Error: {}", e),
            ApiError::ActixWsError(e) => write!(f, "Actix Web Socket Error: {}", e),
            ApiError::WebDataError(e) => write!(f, "Web Data Error: {}", e),
        }
    }
}

impl ResponseError for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::HttpRequestError(err)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::IoError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::JsonError(err)
    }
}

impl From<actix_web::Error> for ApiError {
    fn from(err: actix_web::Error) -> Self {
        ApiError::ActixWsError(err)
    }
}

impl From<std::sync::PoisonError<Servers>> for ApiError {
    fn from(err: std::sync::PoisonError<Servers>) -> Self {
        ApiError::WebDataError(err)
    }
}
