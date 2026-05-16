use std::sync::Arc;

use arc_swap::ArcSwap;
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};

use crate::auth_context::AuthContext;

pub struct AppState {
    pub auth: ArcSwap<AuthContext>,
    pub cipher: Arc<SharedCipher>,
}

impl AppState {
    pub async fn new(auth: AuthContext) -> Result<Self, yt_cipher::Error> {
        Ok(Self {
            auth: ArcSwap::from_pointee(auth),
            cipher: Arc::new(SharedCipher::new().await?),
        })
    }
}

#[derive(Clone)]
pub struct SharedCipher {
    command_tx: mpsc::Sender<CipherCommand>,
}

impl SharedCipher {
    async fn new() -> Result<Self, yt_cipher::Error> {
        let (command_tx, mut command_rx) = mpsc::channel(8);
        let (ready_tx, ready_rx) = oneshot::channel();

        std::thread::spawn(move || {
            let runtime = Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("cipher runtime build failed");

            runtime.block_on(async move {
                match yt_cipher::YtCipher::create().await {
                    Ok(cipher) => {
                        let _ = ready_tx.send(Ok(()));
                        run_cipher_loop(cipher, &mut command_rx).await;
                    }
                    Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            });
        });

        ready_rx
            .await
            .expect("cipher worker thread stopped before initialization")?;

        Ok(Self { command_tx })
    }

    pub async fn signature_timestamp(&self) -> u32 {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::SignatureTimestamp { reply_tx })
            .await
            .expect("cipher worker thread stopped");
        reply_rx.await.expect("cipher worker thread stopped")
    }

    pub async fn refresh(&self) -> Result<(), yt_cipher::Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::Refresh { reply_tx })
            .await
            .expect("cipher worker thread stopped");
        reply_rx.await.expect("cipher worker thread stopped")
    }

    pub async fn decipher(&self, raw: &str) -> Result<String, yt_cipher::Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.command_tx
            .send(CipherCommand::Decipher {
                raw: raw.to_owned(),
                reply_tx,
            })
            .await
            .expect("cipher worker thread stopped");
        reply_rx.await.expect("cipher worker thread stopped")
    }
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

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn app_state_is_send_and_sync() {
        assert_send_sync::<AppState>();
    }

    #[test]
    fn shared_cipher_is_send_and_sync() {
        assert_send_sync::<SharedCipher>();
    }
}
