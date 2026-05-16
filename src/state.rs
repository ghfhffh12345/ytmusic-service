use std::{future::Future, pin::Pin, sync::Arc};

use arc_swap::ArcSwap;
use tokio::runtime::Builder;
use tokio::sync::{Mutex, mpsc, oneshot};

use crate::auth_context::AuthContext;
use crate::error::ServiceError;

pub struct AppState {
    pub auth: ArcSwap<AuthContext>,
    pub cipher: Arc<SharedCipher>,
    reload_lock: Mutex<()>,
    reload_validator: ReloadValidator,
}

impl AppState {
    pub async fn new(auth: AuthContext) -> Result<Self, ServiceError> {
        Ok(Self {
            auth: ArcSwap::from_pointee(auth),
            cipher: Arc::new(SharedCipher::new().await?),
            reload_lock: Mutex::new(()),
            reload_validator: ReloadValidator::Production,
        })
    }

    pub async fn reload_browser_auth(
        &self,
        config: &crate::config::ServiceConfig,
    ) -> Result<String, ServiceError> {
        let _reload_guard = self.reload_lock.lock().await;
        let next = AuthContext::from_browser_auth_file(config).await?;
        self.validate_reload_candidate(&next).await?;
        let version = next.version.to_string();
        self.auth.store(Arc::new(next));
        Ok(version)
    }

    #[doc(hidden)]
    pub fn from_parts_for_tests(auth: AuthContext, cipher: Arc<SharedCipher>) -> Self {
        Self {
            auth: ArcSwap::from_pointee(auth),
            cipher,
            reload_lock: Mutex::new(()),
            reload_validator: ReloadValidator::Production,
        }
    }

    #[doc(hidden)]
    pub fn from_parts_for_reload_tests(
        next: AuthContext,
        cipher: Arc<SharedCipher>,
        validator: Arc<ReloadValidatorFn>,
    ) -> Self {
        Self {
            auth: ArcSwap::from_pointee(next),
            cipher,
            reload_lock: Mutex::new(()),
            reload_validator: ReloadValidator::Injected(validator),
        }
    }

    async fn validate_reload_candidate(&self, next: &AuthContext) -> Result<(), ServiceError> {
        match &self.reload_validator {
            ReloadValidator::Production => next.probe().await,
            ReloadValidator::Injected(validator) => validator(next).await,
        }
    }
}

type ReloadValidationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;
type ReloadValidatorFn =
    dyn for<'a> Fn(&'a AuthContext) -> ReloadValidationFuture<'a> + Send + Sync;

enum ReloadValidator {
    Production,
    Injected(Arc<ReloadValidatorFn>),
}

#[derive(Clone)]
pub struct SharedCipher {
    command_tx: mpsc::Sender<CipherCommand>,
}

impl SharedCipher {
    async fn new() -> Result<Self, ServiceError> {
        let (command_tx, mut command_rx) = mpsc::channel(8);
        let (ready_tx, ready_rx) = oneshot::channel();

        map_thread_spawn_result(
            std::thread::Builder::new()
                .name("ytmusic-cipher-worker".to_owned())
                .spawn(move || {
                    let runtime = match Builder::new_current_thread().enable_all().build() {
                        Ok(runtime) => runtime,
                        Err(error) => {
                            let _ = ready_tx.send(Err(ServiceError::CipherWorkerRuntime(error)));
                            return;
                        }
                    };

                    runtime.block_on(async move {
                        match yt_cipher::YtCipher::create().await {
                            Ok(cipher) => {
                                let _ = ready_tx.send(Ok(()));
                                run_cipher_loop(cipher, &mut command_rx).await;
                            }
                            Err(error) => {
                                let _ = ready_tx.send(Err(ServiceError::CipherWorkerInit(error)));
                            }
                        }
                    });
                }),
        )?;

        ready_rx
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)??;

        Ok(Self { command_tx })
    }

    #[doc(hidden)]
    pub fn unavailable_for_tests() -> Self {
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_rx);
        Self { command_tx }
    }

    pub async fn signature_timestamp(&self) -> Result<u32, ServiceError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::SignatureTimestamp { reply_tx })
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)?;
        reply_rx
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)
    }

    pub async fn refresh(&self) -> Result<(), ServiceError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::Refresh { reply_tx })
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)?;
        reply_rx
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)?
            .map_err(ServiceError::CipherOperation)
    }

    pub async fn decipher(&self, raw: &str) -> Result<String, ServiceError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::Decipher {
                raw: raw.to_owned(),
                reply_tx,
            })
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)?;
        reply_rx
            .await
            .map_err(|_| ServiceError::CipherWorkerUnavailable)?
            .map_err(ServiceError::CipherOperation)
    }
}

fn map_thread_spawn_result(
    spawn_result: std::io::Result<std::thread::JoinHandle<()>>,
) -> Result<(), ServiceError> {
    spawn_result
        .map(|_| ())
        .map_err(ServiceError::CipherWorkerThreadSpawn)
}

enum CipherCommand {
    SignatureTimestamp {
        reply_tx: oneshot::Sender<u32>,
    },
    Refresh {
        reply_tx: oneshot::Sender<Result<(), yt_cipher::Error>>,
    },
    Decipher {
        raw: String,
        reply_tx: oneshot::Sender<Result<String, yt_cipher::Error>>,
    },
}

async fn run_cipher_loop(
    cipher: yt_cipher::YtCipher,
    command_rx: &mut mpsc::Receiver<CipherCommand>,
) {
    while let Some(command) = command_rx.recv().await {
        match command {
            CipherCommand::SignatureTimestamp { reply_tx } => {
                let _ = reply_tx.send(cipher.signature_timestamp());
            }
            CipherCommand::Refresh { reply_tx } => {
                let _ = reply_tx.send(cipher.refresh().await);
            }
            CipherCommand::Decipher { raw, reply_tx } => {
                let _ = reply_tx.send(cipher.decipher(&raw).await);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, SharedCipher};
    use crate::adapters::cipher::CipherAdapter;
    use crate::error::ServiceError;
    use tokio::sync::mpsc;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn app_state_is_send_and_sync() {
        assert_send_sync::<AppState>();
    }

    #[test]
    fn shared_cipher_is_send_and_sync() {
        assert_send_sync::<SharedCipher>();
    }

    #[test]
    fn thread_spawn_failure_maps_to_service_error() {
        let spawn_result = Err(std::io::Error::other("thread spawn failed"));

        let error = super::map_thread_spawn_result(spawn_result).unwrap_err();

        assert!(matches!(error, ServiceError::CipherWorkerThreadSpawn(_)));
    }

    #[tokio::test]
    async fn shared_cipher_returns_service_error_when_worker_channel_is_closed() {
        let (command_tx, command_rx) = mpsc::channel(1);
        drop(command_rx);

        let cipher = SharedCipher { command_tx };

        let error = cipher.signature_timestamp().await.unwrap_err();

        assert!(matches!(error, ServiceError::CipherWorkerUnavailable));
    }

    #[tokio::test]
    async fn cipher_adapter_normalizes_cipher_operation_failures() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let cipher = SharedCipher { command_tx };

        tokio::spawn(async move {
            let command = command_rx.recv().await.expect("decipher command");
            match command {
                super::CipherCommand::Decipher { reply_tx, .. } => {
                    let _ = reply_tx.send(Err(yt_cipher::Error::CipherParse));
                }
                _ => panic!("expected decipher command"),
            }
        });

        let error = CipherAdapter::decipher(&cipher, "raw").await.unwrap_err();

        assert!(matches!(error, ServiceError::Cipher(_)));
    }
}
