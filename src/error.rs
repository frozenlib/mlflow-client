use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{error_code} : {message}")]
    ApiError { error_code: String, message: String },
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Task join failed")]
    TaskJoinError,
    #[error("Error: {0}")]
    Message(String),
}

impl Error {
    pub fn from_message(message: impl Display) -> Error {
        Error::Message(message.to_string())
    }

    pub fn is_resource_does_not_exist(&self) -> bool {
        match self {
            Error::ApiError { error_code, .. } => error_code == "RESOURCE_DOES_NOT_EXIST",
            _ => false,
        }
    }
}
