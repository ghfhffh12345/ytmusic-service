#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("browser auth file does not exist: {0}")]
    BrowserAuthPathMissing(std::path::PathBuf),
    #[error("browser auth path is not a regular file: {0}")]
    BrowserAuthPathNotFile(std::path::PathBuf),
    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] std::net::AddrParseError),
    #[error("failed to load browser auth: {0}")]
    BrowserAuthLoad(#[source] ytmusicapi::Error),
    #[error("failed to spawn cipher worker thread: {0}")]
    CipherWorkerThreadSpawn(#[source] std::io::Error),
    #[error("failed to build cipher worker runtime: {0}")]
    CipherWorkerRuntime(#[source] std::io::Error),
    #[error("failed to initialize cipher worker: {0}")]
    CipherWorkerInit(#[source] yt_cipher::Error),
    #[error("cipher worker is unavailable")]
    CipherWorkerUnavailable,
    #[error("cipher operation failed: {0}")]
    CipherOperation(#[source] yt_cipher::Error),
}
