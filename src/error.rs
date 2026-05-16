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
    #[error("ytmusicapi request failed: {0}")]
    YtMusic(#[source] ytmusicapi::Error),
    #[error("failed to spawn cipher worker thread: {0}")]
    CipherWorkerThreadSpawn(#[source] std::io::Error),
    #[error("failed to build cipher worker runtime: {0}")]
    CipherWorkerRuntime(#[source] std::io::Error),
    #[error("failed to initialize cipher worker: {0}")]
    CipherWorkerInit(#[source] yt_cipher::Error),
    #[error("yt-cipher request failed: {0}")]
    Cipher(#[source] yt_cipher::Error),
    #[error("cipher worker is unavailable")]
    CipherWorkerUnavailable,
    #[error("cipher operation failed: {0}")]
    CipherOperation(#[source] yt_cipher::Error),
}

pub fn map_invalid_argument(message: impl Into<String>) -> tonic::Status {
    tonic::Status::invalid_argument(message.into())
}

pub fn map_service_error(error: &ServiceError) -> tonic::Status {
    match error {
        ServiceError::YtMusic(source) => tonic::Status::unavailable(source.to_string()),
        ServiceError::Cipher(source) => tonic::Status::internal(source.to_string()),
        ServiceError::BrowserAuthPathMissing(path) => tonic::Status::failed_precondition(format!(
            "browser auth file missing: {}",
            path.display()
        )),
        _ => tonic::Status::internal(error.to_string()),
    }
}
