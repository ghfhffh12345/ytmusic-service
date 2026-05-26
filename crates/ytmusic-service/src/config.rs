use std::{net::SocketAddr, path::PathBuf, time::Duration};

use crate::error::ServiceError;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    listen_addr: SocketAddr,
    browser_auth_path: PathBuf,
    rpc_timeout: Option<Duration>,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self, ServiceError> {
        let listen_addr =
            std::env::var("YTMUSIC_SERVICE_ADDR").map_err(|source| ServiceError::EnvVar {
                name: "YTMUSIC_SERVICE_ADDR",
                source,
            })?;
        let browser_auth_path =
            std::env::var("YTMUSIC_SERVICE_BROWSER_JSON").map_err(|source| {
                ServiceError::EnvVar {
                    name: "YTMUSIC_SERVICE_BROWSER_JSON",
                    source,
                }
            })?;
        let rpc_timeout = std::env::var("YTMUSIC_SERVICE_RPC_TIMEOUT_MS")
            .ok()
            .map(|value| parse_rpc_timeout_ms(&value))
            .transpose()?;

        let mut config = Self::from_parts(&listen_addr, PathBuf::from(browser_auth_path))?;
        config.rpc_timeout = rpc_timeout;
        Ok(config)
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
            rpc_timeout: None,
        })
    }

    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn browser_auth_path(&self) -> &std::path::Path {
        &self.browser_auth_path
    }

    pub fn rpc_timeout(&self) -> Option<Duration> {
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

fn parse_rpc_timeout_ms(value: &str) -> Result<Duration, ServiceError> {
    let millis = value
        .parse::<u64>()
        .map_err(|source| ServiceError::InvalidRpcTimeoutMs {
            value: value.to_owned(),
            reason: source.to_string(),
        })?;
    if millis == 0 {
        return Err(ServiceError::InvalidRpcTimeoutMs {
            value: value.to_owned(),
            reason: "must be a positive integer greater than 0".to_owned(),
        });
    }
    Ok(Duration::from_millis(millis))
}

#[cfg(test)]
mod tests {
    use super::parse_rpc_timeout_ms;
    use crate::error::ServiceError;

    #[test]
    fn parse_rpc_timeout_ms_rejects_zero() {
        let error = parse_rpc_timeout_ms("0").unwrap_err();

        match error {
            ServiceError::InvalidRpcTimeoutMs { value, reason } => {
                assert_eq!(value, "0");
                assert_eq!(reason, "must be a positive integer greater than 0");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
