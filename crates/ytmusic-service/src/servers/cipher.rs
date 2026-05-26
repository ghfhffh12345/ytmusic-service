use std::sync::Arc;

use tonic::{Request, Response, Status};
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_cipher_server::YtCipher};

const UNIMPLEMENTED_MESSAGE: &str = "ytmusic.v2.YtCipher RPCs are not implemented yet";

pub struct CipherService {
    pub state: Arc<crate::state::AppState>,
}

#[tonic::async_trait]
impl YtCipher for CipherService {
    async fn get_signature_timestamp(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::GetSignatureTimestampResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn refresh(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::RefreshCipherResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }

    async fn decipher(
        &self,
        _request: Request<pb::DecipherRequest>,
    ) -> Result<Response<pb::DecipherResponse>, Status> {
        Err(Status::unimplemented(UNIMPLEMENTED_MESSAGE))
    }
}
