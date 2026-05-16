use std::{
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
            version: Arc::<str>::from(version_from_loaded_at(loaded_at)),
            loaded_at,
        })
    }
}

fn version_from_loaded_at(loaded_at: SystemTime) -> String {
    let timestamp = loaded_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = AUTH_CONTEXT_VERSION_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{timestamp:032x}-{sequence:016x}")
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::version_from_loaded_at;

    #[test]
    fn version_generation_handles_pre_epoch_clock() {
        let version = version_from_loaded_at(UNIX_EPOCH - Duration::from_secs(1));

        assert!(!version.is_empty());
    }
}
