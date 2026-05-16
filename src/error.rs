#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("browser auth file does not exist: {0}")]
    BrowserAuthPathMissing(std::path::PathBuf),
    #[error("browser auth path is not a regular file: {0}")]
    BrowserAuthPathNotFile(std::path::PathBuf),
    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] std::net::AddrParseError),
}
