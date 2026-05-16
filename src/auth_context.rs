use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{config::ServiceConfig, error::ServiceError};

static AUTH_CONTEXT_VERSION_SEQ: AtomicU64 = AtomicU64::new(0);

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
            version: Arc::<str>::from(version_from_path(config.browser_auth_path(), loaded_at)),
            loaded_at,
        })
    }
}

fn version_from_path(path: &Path, loaded_at: SystemTime) -> String {
    let timestamp = loaded_at
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let sequence = AUTH_CONTEXT_VERSION_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}:{timestamp}:{sequence}", path.display())
}
