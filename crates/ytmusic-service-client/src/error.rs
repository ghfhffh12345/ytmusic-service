const SUBSYSTEM_METADATA_KEY: &str = "ytmusic-service-subsystem";

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ClientStatus(tonic::Status);

impl ClientStatus {
    pub fn code(&self) -> tonic::Code {
        self.0.code()
    }

    pub fn message(&self) -> &str {
        self.0.message()
    }

    pub fn subsystem(&self) -> Option<&str> {
        self.0
            .metadata()
            .get(SUBSYSTEM_METADATA_KEY)
            .and_then(|value| value.to_str().ok())
    }

    pub fn is_invalid_argument(&self) -> bool {
        self.code() == tonic::Code::InvalidArgument
    }

    pub fn is_failed_precondition(&self) -> bool {
        self.code() == tonic::Code::FailedPrecondition
    }

    pub fn is_unavailable(&self) -> bool {
        self.code() == tonic::Code::Unavailable
    }

    pub fn is_internal(&self) -> bool {
        self.code() == tonic::Code::Internal
    }

    pub fn is_unimplemented(&self) -> bool {
        self.code() == tonic::Code::Unimplemented
    }
}

impl From<tonic::Status> for ClientStatus {
    fn from(status: tonic::Status) -> Self {
        Self(status)
    }
}
