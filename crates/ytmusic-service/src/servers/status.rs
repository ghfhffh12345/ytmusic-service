use std::{sync::Arc, time::SystemTime};

use tonic::{Request, Response, Status};
use ytmusic_service_proto::ytmusic::v2::{self as pb, service_status_server::ServiceStatus};

pub struct StatusService {
    pub state: Arc<crate::state::AppState>,
}

#[tonic::async_trait]
impl ServiceStatus for StatusService {
    async fn get_status(
        &self,
        _request: Request<pb::GetStatusRequest>,
    ) -> Result<Response<pb::GetStatusResponse>, Status> {
        let started_at_unix_seconds = self
            .state
            .started_at
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(Response::new(pb::GetStatusResponse {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            started_at_unix_seconds,
            listen_addr: self.state.listen_addr.to_string(),
            ytmusic_ready: true,
            cipher_ready: true,
            lifecycle: "serving".to_owned(),
        }))
    }
}
