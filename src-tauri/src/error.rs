use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("vault not found: {0}")]
    VaultNotFound(String),

    #[error("vault already exists: {0}")]
    VaultAlreadyExists(String),

    #[error("no vault is currently open")]
    NoVaultOpen,

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("io error: {0}")]
    IoError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("index error: {0}")]
    IndexError(String),

    #[error("search error: {0}")]
    SearchError(String),

    #[error("watcher error: {0}")]
    WatcherError(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::IoError(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::InternalError(e.to_string())
    }
}

impl From<tantivy::TantivyError> for AppError {
    fn from(e: tantivy::TantivyError) -> Self {
        AppError::SearchError(e.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(e: regex::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::ParseError(e.to_string())
    }
}

impl From<notify::Error> for AppError {
    fn from(e: notify::Error) -> Self {
        AppError::WatcherError(e.to_string())
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(e: std::time::SystemTimeError) -> Self {
        AppError::IoError(e.to_string())
    }
}

impl From<std::path::StripPrefixError> for AppError {
    fn from(e: std::path::StripPrefixError) -> Self {
        AppError::InvalidPath(e.to_string())
    }
}

impl From<walkdir::Error> for AppError {
    fn from(e: walkdir::Error) -> Self {
        AppError::IoError(e.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for AppError {
    fn from(e: tantivy::query::QueryParserError) -> Self {
        AppError::SearchError(e.to_string())
    }
}
