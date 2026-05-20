use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{config::ServiceConfig, state::AppState};
use ytmusic_service_proto::ytmusic::v1::admin::{
    ReloadBrowserAuthRequest, ReloadBrowserAuthResponse, yt_music_admin_server::YtMusicAdmin,
};

pub struct AdminService {
    pub state: Arc<AppState>,
    pub config: ServiceConfig,
}

#[tonic::async_trait]
impl YtMusicAdmin for AdminService {
    async fn reload_browser_auth(
        &self,
        _request: Request<ReloadBrowserAuthRequest>,
    ) -> Result<Response<ReloadBrowserAuthResponse>, Status> {
        let version = self
            .state
            .reload_browser_auth(&self.config)
            .await
            .map_err(|error| crate::error::map_service_error(&error))?;

        Ok(Response::new(ReloadBrowserAuthResponse {
            active_version: version,
        }))
    }
}
