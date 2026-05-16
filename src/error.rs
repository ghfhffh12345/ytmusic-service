use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("missing or invalid environment variable {name}: {source}")]
    EnvVar {
        name: &'static str,
        #[source]
        source: std::env::VarError,
    },
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
    #[error("failed to build server reflection: {0}")]
    Reflection(#[source] tonic_reflection::server::Error),
    #[error("transport failure: {0}")]
    Transport(#[source] tonic::transport::Error),
}

pub fn map_invalid_argument(message: impl Into<String>) -> tonic::Status {
    tonic::Status::invalid_argument(message.into())
}

pub fn map_service_error(error: &ServiceError) -> tonic::Status {
    match error {
        ServiceError::EnvVar { source, .. } => tonic::Status::failed_precondition(source.to_string()),
        ServiceError::BrowserAuthPathMissing(path) => tonic::Status::failed_precondition(format!(
            "browser auth file missing: {}",
            path.display()
        )),
        ServiceError::BrowserAuthPathNotFile(path) => tonic::Status::failed_precondition(format!(
            "browser auth path is not a file: {}",
            path.display()
        )),
        ServiceError::InvalidSocketAddress(source) => {
            tonic::Status::failed_precondition(source.to_string())
        }
        ServiceError::BrowserAuthLoad(source) => map_ytmusic_error(source),
        ServiceError::YtMusic(source) => map_ytmusic_error(source),
        ServiceError::CipherWorkerThreadSpawn(source) => {
            tonic::Status::unavailable(source.to_string())
        }
        ServiceError::CipherWorkerRuntime(source) => tonic::Status::unavailable(source.to_string()),
        ServiceError::CipherWorkerInit(source) => map_cipher_error(source),
        ServiceError::Cipher(source) => map_cipher_error(source),
        ServiceError::CipherWorkerUnavailable => tonic::Status::unavailable(error.to_string()),
        ServiceError::CipherOperation(source) => map_cipher_error(source),
        ServiceError::Reflection(source) => tonic::Status::internal(source.to_string()),
        ServiceError::Transport(source) => tonic::Status::unavailable(source.to_string()),
    }
}

fn map_ytmusic_error(error: &ytmusicapi::Error) -> tonic::Status {
    match error {
        ytmusicapi::Error::InvalidInput(message) => {
            tonic::Status::invalid_argument(message.clone())
        }
        ytmusicapi::Error::AuthValidation(message) => {
            tonic::Status::failed_precondition(message.clone())
        }
        ytmusicapi::Error::AuthFileRead { source, .. } => {
            tonic::Status::failed_precondition(source.to_string())
        }
        ytmusicapi::Error::AuthFileDecode { source, .. } => {
            tonic::Status::failed_precondition(source.to_string())
        }
        ytmusicapi::Error::HttpClientBuild(source) => tonic::Status::internal(source.to_string()),
        ytmusicapi::Error::HttpTransport(source) => tonic::Status::unavailable(source.to_string()),
        ytmusicapi::Error::HttpStatus { status, message } => map_http_status(*status, message),
        ytmusicapi::Error::JsonDecode(source) => tonic::Status::internal(source.to_string()),
        ytmusicapi::Error::MissingVisitorId => tonic::Status::unavailable(error.to_string()),
        ytmusicapi::Error::MissingBootstrapField(field) => tonic::Status::unavailable(format!(
            "failed to bootstrap anonymous client config: missing {field}"
        )),
        ytmusicapi::Error::Parse(message) => tonic::Status::internal(message.clone()),
        ytmusicapi::Error::UnsupportedFeature(message) => {
            tonic::Status::unimplemented(message.clone())
        }
    }
}

fn map_cipher_error(error: &yt_cipher::Error) -> tonic::Status {
    match error {
        yt_cipher::Error::CipherParse => tonic::Status::invalid_argument(error.to_string()),
        yt_cipher::Error::NTransformUnavailable | yt_cipher::Error::SignatureSolverUnavailable => {
            tonic::Status::unimplemented(error.to_string())
        }
        yt_cipher::Error::HomepageFetch(_)
        | yt_cipher::Error::PlayerFetch(_)
        | yt_cipher::Error::QuickJsRuntimeInitFailed => {
            tonic::Status::unavailable(error.to_string())
        }
        _ => tonic::Status::internal(error.to_string()),
    }
}

fn map_http_status(status: StatusCode, message: &str) -> tonic::Status {
    match status {
        StatusCode::BAD_REQUEST => tonic::Status::invalid_argument(message.to_owned()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            tonic::Status::failed_precondition(message.to_owned())
        }
        StatusCode::NOT_FOUND => tonic::Status::not_found(message.to_owned()),
        StatusCode::TOO_MANY_REQUESTS => tonic::Status::resource_exhausted(message.to_owned()),
        _ if status.is_server_error() => tonic::Status::unavailable(message.to_owned()),
        _ => tonic::Status::unknown(message.to_owned()),
    }
}
