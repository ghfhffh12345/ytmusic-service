use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::auth_context::AuthContext;

pub struct AppState {
    pub auth: ArcSwap<AuthContext>,
    pub cipher: Arc<yt_cipher::YtCipher>,
}

impl AppState {
    pub fn new(auth: AuthContext, cipher: yt_cipher::YtCipher) -> Self {
        Self {
            auth: ArcSwap::from_pointee(auth),
            cipher: Arc::new(cipher),
        }
    }
}
