use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("Path traversal security violation: {0}")]
    TraversalAttempt(String),
    #[error("Frontmatter parsing error: {0}")]
    Frontmatter(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

