use std::{net::SocketAddr, path::PathBuf};

use crate::error::ServiceError;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    public_addr: SocketAddr,
    admin_addr: SocketAddr,
    browser_auth_path: PathBuf,
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
        if !browser_auth_path.is_file() {
            return Err(ServiceError::BrowserAuthPathNotFile(browser_auth_path));
        }

        Ok(Self {
            public_addr: public_addr.parse()?,
            admin_addr: admin_addr.parse()?,
            browser_auth_path,
        })
    }

    pub fn public_addr(&self) -> SocketAddr {
        self.public_addr
    }

    pub fn admin_addr(&self) -> SocketAddr {
        self.admin_addr
    }

    pub fn browser_auth_path(&self) -> &std::path::Path {
        &self.browser_auth_path
    }
}
