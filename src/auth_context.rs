use std::{path::Path, sync::Arc, time::SystemTime};

use crate::{config::ServiceConfig, error::ServiceError};

#[derive(Clone)]
pub struct AuthContext {
    pub client: ytmusicapi::YtMusic,
    pub version: Arc<str>,
    pub loaded_at: SystemTime,
}

impl AuthContext {
    pub async fn from_browser_auth_file(config: &ServiceConfig) -> Result<Self, ServiceError> {
        let client = ytmusicapi::YtMusic::from_browser_auth_file(config.browser_auth_path())
            .map_err(ServiceError::BrowserAuthLoad)?;

        Ok(Self {
            client,
            version: Arc::<str>::from(version_from_path(config.browser_auth_path())),
            loaded_at: SystemTime::now(),
        })
    }
}

fn version_from_path(path: &Path) -> String {
    format!("{}:{}", path.display(), chrono::Utc::now().timestamp())
}
