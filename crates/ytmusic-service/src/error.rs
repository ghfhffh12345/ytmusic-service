use reqwest::StatusCode;

const SUBSYSTEM_METADATA_KEY: &str = "ytmusic-service-subsystem";

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
    #[error("invalid YTMUSIC_SERVICE_RPC_TIMEOUT_MS value {value}: {reason}")]
    InvalidRpcTimeoutMs { value: String, reason: String },
    #[error("failed to bind listener: {0}")]
    ListenerBind(#[source] std::io::Error),
    #[error("failed to read listener local address: {0}")]
    ListenerLocalAddr(#[source] std::io::Error),
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
    #[error("failed to prepare listener incoming stream: {0}")]
    Incoming(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("transport failure: {0}")]
    Transport(#[source] tonic::transport::Error),
    #[error("test server failed to signal readiness")]
    TestServerReadySignal,
}

pub fn map_invalid_argument(subsystem: &str, message: impl Into<String>) -> tonic::Status {
    status_with_subsystem(subsystem, tonic::Code::InvalidArgument, message)
}

pub fn map_service_error(subsystem: &str, error: &ServiceError) -> tonic::Status {
    match error {
        ServiceError::EnvVar { source, .. } => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            source.to_string(),
        ),
        ServiceError::BrowserAuthPathMissing(path) => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            format!("browser auth file missing: {}", path.display()),
        ),
        ServiceError::BrowserAuthPathNotFile(path) => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            format!("browser auth path is not a file: {}", path.display()),
        ),
        ServiceError::InvalidSocketAddress(source) => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            source.to_string(),
        ),
        ServiceError::InvalidRpcTimeoutMs { reason, .. } => {
            status_with_subsystem(subsystem, tonic::Code::FailedPrecondition, reason.clone())
        }
        ServiceError::ListenerBind(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::ListenerLocalAddr(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::BrowserAuthLoad(source) => map_ytmusic_error(subsystem, source),
        ServiceError::YtMusic(source) => map_ytmusic_error(subsystem, source),
        ServiceError::CipherWorkerThreadSpawn(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::CipherWorkerRuntime(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::CipherWorkerInit(source) => map_cipher_error(subsystem, source),
        ServiceError::Cipher(source) => map_cipher_error(subsystem, source),
        ServiceError::CipherWorkerUnavailable => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, error.to_string())
        }
        ServiceError::CipherOperation(source) => map_cipher_error(subsystem, source),
        ServiceError::Reflection(source) => {
            status_with_subsystem(subsystem, tonic::Code::Internal, source.to_string())
        }
        ServiceError::Incoming(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::Transport(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ServiceError::TestServerReadySignal => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, error.to_string())
        }
    }
}

fn map_ytmusic_error(subsystem: &str, error: &ytmusicapi::Error) -> tonic::Status {
    match error {
        ytmusicapi::Error::InvalidInput(message) => {
            status_with_subsystem(subsystem, tonic::Code::InvalidArgument, message.clone())
        }
        ytmusicapi::Error::AuthValidation(message) => {
            status_with_subsystem(subsystem, tonic::Code::FailedPrecondition, message.clone())
        }
        ytmusicapi::Error::AuthFileRead { source, .. } => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            source.to_string(),
        ),
        ytmusicapi::Error::AuthFileDecode { source, .. } => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            source.to_string(),
        ),
        ytmusicapi::Error::HttpClientBuild(source) => {
            status_with_subsystem(subsystem, tonic::Code::Internal, source.to_string())
        }
        ytmusicapi::Error::HttpTransport(source) => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, source.to_string())
        }
        ytmusicapi::Error::HttpStatus { status, message } => {
            map_http_status(subsystem, *status, message)
        }
        ytmusicapi::Error::JsonDecode(source) => {
            status_with_subsystem(subsystem, tonic::Code::Internal, source.to_string())
        }
        ytmusicapi::Error::MissingVisitorId => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, error.to_string())
        }
        ytmusicapi::Error::MissingBootstrapField(field) => status_with_subsystem(
            subsystem,
            tonic::Code::Unavailable,
            format!("failed to bootstrap anonymous client config: missing {field}"),
        ),
        ytmusicapi::Error::Parse(message) => {
            status_with_subsystem(subsystem, tonic::Code::Internal, message.clone())
        }
        ytmusicapi::Error::UnsupportedFeature(message) => {
            status_with_subsystem(subsystem, tonic::Code::Unimplemented, message.clone())
        }
    }
}

fn map_cipher_error(subsystem: &str, error: &yt_cipher::Error) -> tonic::Status {
    match error {
        yt_cipher::Error::CipherParse => {
            status_with_subsystem(subsystem, tonic::Code::InvalidArgument, error.to_string())
        }
        yt_cipher::Error::NTransformUnavailable | yt_cipher::Error::SignatureSolverUnavailable => {
            status_with_subsystem(subsystem, tonic::Code::Unimplemented, error.to_string())
        }
        yt_cipher::Error::HomepageFetch(_)
        | yt_cipher::Error::PlayerFetch(_)
        | yt_cipher::Error::QuickJsRuntimeInitFailed => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, error.to_string())
        }
        _ => status_with_subsystem(subsystem, tonic::Code::Internal, error.to_string()),
    }
}

fn map_http_status(subsystem: &str, status: StatusCode, message: &str) -> tonic::Status {
    match status {
        StatusCode::BAD_REQUEST => {
            status_with_subsystem(subsystem, tonic::Code::InvalidArgument, message.to_owned())
        }
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => status_with_subsystem(
            subsystem,
            tonic::Code::FailedPrecondition,
            message.to_owned(),
        ),
        StatusCode::NOT_FOUND => {
            status_with_subsystem(subsystem, tonic::Code::NotFound, message.to_owned())
        }
        StatusCode::TOO_MANY_REQUESTS => status_with_subsystem(
            subsystem,
            tonic::Code::ResourceExhausted,
            message.to_owned(),
        ),
        _ if status.is_server_error() => {
            status_with_subsystem(subsystem, tonic::Code::Unavailable, message.to_owned())
        }
        _ => status_with_subsystem(subsystem, tonic::Code::Unknown, message.to_owned()),
    }
}

fn status_with_subsystem(
    subsystem: &str,
    code: tonic::Code,
    message: impl Into<String>,
) -> tonic::Status {
    let mut status = tonic::Status::new(code, message.into());
    if let Ok(value) = subsystem.parse() {
        status.metadata_mut().insert(SUBSYSTEM_METADATA_KEY, value);
    }
    status
}
