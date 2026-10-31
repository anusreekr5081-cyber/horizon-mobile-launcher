use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LauncherError>;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid profile: {0}")]
    InvalidProfile(String),

    #[error("invalid version manifest: {0}")]
    InvalidManifest(String),

    #[error("file checksum did not match for {path}")]
    ChecksumMismatch { path: PathBuf },

    #[error("required file is missing: {0}")]
    MissingFile(PathBuf),
}
