use std::{net::SocketAddr, path::PathBuf, time::Duration};

use crate::error::ServiceError;

const DEFAULT_RPC_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    listen_addr: SocketAddr,
    browser_auth_path: PathBuf,
    rpc_timeout: Duration,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ServiceError> {
        let listen_addr = std::env::var("YTMUSIC_SERVICE_LISTEN_ADDR").map_err(|source| {
            ServiceError::EnvVar {
                name: "YTMUSIC_SERVICE_LISTEN_ADDR",
                source,
            }
        })?;
        let browser_auth_path =
            std::env::var("YTMUSIC_SERVICE_BROWSER_JSON").map_err(|source| {
                ServiceError::EnvVar {
                    name: "YTMUSIC_SERVICE_BROWSER_JSON",
                    source,
                }
            })?;

        Self::from_parts(&listen_addr, PathBuf::from(browser_auth_path))
    }

    pub fn from_parts(
        listen_addr: &str,
        browser_auth_path: impl Into<PathBuf>,
    ) -> Result<Self, ServiceError> {
        let browser_auth_path = browser_auth_path.into();
        if !browser_auth_path.exists() {
            return Err(ServiceError::BrowserAuthPathMissing(browser_auth_path));
        }
        if !browser_auth_path.is_file() {
            return Err(ServiceError::BrowserAuthPathNotFile(browser_auth_path));
        }

        Ok(Self {
            listen_addr: listen_addr.parse()?,
            browser_auth_path,
            rpc_timeout: DEFAULT_RPC_TIMEOUT,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn browser_auth_path(&self) -> &std::path::Path {
        &self.browser_auth_path
    }

    pub fn rpc_timeout(&self) -> Duration {
        self.rpc_timeout
    }

    pub(crate) fn with_listen_addr(&self, listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            browser_auth_path: self.browser_auth_path.clone(),
            rpc_timeout: self.rpc_timeout,
        }
    }
}
