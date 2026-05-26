use std::sync::Arc;

use tonic::{Request, Response, Status};
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_cipher_server::YtCipher};

pub struct CipherService {
    pub state: Arc<crate::state::AppState>,
}

#[tonic::async_trait]
impl YtCipher for CipherService {
    async fn get_signature_timestamp(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::GetSignatureTimestampResponse>, Status> {
        let signature_timestamp = self
            .state
            .cipher
            .signature_timestamp()
            .await
            .map_err(|source| crate::error::map_service_error("cipher", &source))?;

        Ok(Response::new(pb::GetSignatureTimestampResponse {
            signature_timestamp,
        }))
    }

    async fn refresh(
        &self,
        _request: Request<pb::Empty>,
    ) -> Result<Response<pb::RefreshCipherResponse>, Status> {
        self.state
            .cipher
            .refresh()
            .await
            .map_err(|source| crate::error::map_service_error("cipher", &source))?;

        Ok(Response::new(pb::RefreshCipherResponse {}))
    }

    async fn decipher(
        &self,
        request: Request<pb::DecipherRequest>,
    ) -> Result<Response<pb::DecipherResponse>, Status> {
        let request = request.into_inner();
        if request.signature_cipher.trim().is_empty() {
            return Err(crate::error::map_invalid_argument(
                "cipher",
                "signature_cipher must not be empty",
            ));
        }

        let playable_url = self
            .state
            .cipher
            .decipher(&request.signature_cipher)
            .await
            .map_err(|source| crate::error::map_service_error("cipher", &source))?;

        Ok(Response::new(pb::DecipherResponse { playable_url }))
    }
}
