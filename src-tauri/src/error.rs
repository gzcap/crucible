//! 错误类型定义
//!
//! 定义应用中使用的所有错误类型，以及从标准库和第三方库错误到应用错误的转换。
//!
//! 使用 `thiserror` 宏自动实现 `std::error::Error` trait，并通过 `serde` 支持序列化到前端。

use serde::Serialize;
use thiserror::Error;

/// 应用错误枚举
///
/// 定义所有可能的错误类型，使用 `#[serde(tag = "type", content = "message")]` 确保序列化后
/// 前端可以根据 `type` 字段识别错误类型，`message` 字段包含详细错误信息。
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    /// Vault 未找到
    #[error("vault not found: {0}")]
    VaultNotFound(String),

    /// Vault 已存在
    #[error("vault already exists: {0}")]
    VaultAlreadyExists(String),

    /// 当前没有打开的 Vault
    #[error("no vault is currently open")]
    NoVaultOpen,

    /// 文件未找到
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// IO 错误
    #[error("io error: {0}")]
    IoError(String),

    /// 解析错误
    #[error("parse error: {0}")]
    ParseError(String),

    /// 索引错误
    #[error("index error: {0}")]
    IndexError(String),

    /// 搜索错误
    #[error("search error: {0}")]
    SearchError(String),

    /// 文件监控错误
    #[error("watcher error: {0}")]
    WatcherError(String),

    /// 无效路径
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// 内部错误
    #[error("internal error: {0}")]
    InternalError(String),

    /// 插件未找到
    #[error("plugin not found: {0}")]
    PluginNotFound(String),

    /// 插件加载错误
    #[error("plugin load error: {0}: {1}")]
    PluginLoadError(String, String),

    /// 插件权限不足
    #[error("plugin permission denied: {0}")]
    PluginPermissionDenied(String),
}

/// 应用结果类型别名
///
/// 简化错误处理，所有返回结果的函数都使用此类型。
pub type Result<T> = std::result::Result<T, AppError>;

// ========== 错误转换实现 ==========

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

impl From<fs_extra::error::Error> for AppError {
    fn from(e: fs_extra::error::Error) -> Self {
        AppError::IoError(e.to_string())
    }
}