use std::{sync::Arc, time::SystemTime};

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
        let loaded_at = SystemTime::now();

        Ok(Self {
            client,
            version: Arc::<str>::from(new_version_token()),
            loaded_at,
        })
    }

    pub async fn probe(&self) -> Result<(), ServiceError> {
        self.client
            .get_account_info()
            .await
            .map(|_| ())
            .map_err(ServiceError::YtMusic)
    }
}

fn new_version_token() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}
