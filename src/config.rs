use std::{net::SocketAddr, path::PathBuf};

use crate::error::ServiceError;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub public_addr: SocketAddr,
    pub admin_addr: SocketAddr,
    pub browser_auth_path: PathBuf,
}

impl ServiceConfig {
    pub fn from_parts(
        public_addr: &str,
        admin_addr: &str,
        browser_auth_path: PathBuf,
    ) -> Result<Self, ServiceError> {
        if !browser_auth_path.exists() {
            return Err(ServiceError::BrowserAuthPathMissing(browser_auth_path));
        }

        Ok(Self {
            public_addr: public_addr.parse()?,
            admin_addr: admin_addr.parse()?,
            browser_auth_path,
        })
    }
}
