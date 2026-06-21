use thiserror::Error;

#[derive(Debug, Error)]
pub enum XbpsError {
    #[error("xbps-query not found. Is xbps installed?")]
    NotFound,

    #[error("Failed to run xbps-query: {0}")]
    Io(#[from] std::io::Error),

    #[error("Package '{0}' not found")]
    PackageNotFound(String),
}
