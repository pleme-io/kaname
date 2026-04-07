//! Typed error variants for kaname operations.

/// Errors that can occur in kaname MCP server operations.
#[derive(Debug, thiserror::Error)]
pub enum KanameError {
    /// A requested configuration key was not found.
    #[error("config key not found: {key}")]
    ConfigKeyNotFound {
        /// The key that was looked up.
        key: String,
    },

    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An I/O error occurred (e.g. during transport setup).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
