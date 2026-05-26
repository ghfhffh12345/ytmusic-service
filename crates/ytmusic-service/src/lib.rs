pub mod config;
pub mod error;
pub mod servers;
pub mod state;

use std::{net::SocketAddr, sync::Arc};

use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use ytmusic_service_proto::ytmusic::v2::{
    FILE_DESCRIPTOR_SET, service_status_server::ServiceStatusServer,
    yt_cipher_server::YtCipherServer, yt_music_server::YtMusicServer,
};

pub async fn run(config: config::ServiceConfig) -> Result<(), error::ServiceError> {
    let listener = bind_service_listener(config.listen_addr()).await?;
    let local_addr = listener
        .local_addr()
        .map_err(error::ServiceError::ListenerLocalAddr)?;
    let state = Arc::new(state::AppState::new(&config.with_listen_addr(local_addr)).await?);

    serve(listener, state, config.rpc_timeout(), None).await
}

#[doc(hidden)]
pub async fn run_for_tests(
    config: config::ServiceConfig,
) -> Result<TestHarness, error::ServiceError> {
    let listener = bind_service_listener(config.listen_addr()).await?;
    let local_addr = listener
        .local_addr()
        .map_err(error::ServiceError::ListenerLocalAddr)?;
    let music = ytmusicapi::YtMusic::builder()
        .browser_auth_path(config.browser_auth_path().to_path_buf())
        .build()
        .map_err(error::ServiceError::BrowserAuthLoad)?;
    let state = Arc::new(state::AppState::from_parts_for_tests(
        music,
        state::SharedCipher::unavailable_for_tests(),
        std::time::SystemTime::now(),
        local_addr,
    ));
    let rpc_timeout = config.rpc_timeout();
    let (ready_tx, ready_rx) = oneshot::channel();
    let server =
        tokio::spawn(async move { serve(listener, state, rpc_timeout, Some(ready_tx)).await });

    ready_rx
        .await
        .map_err(|_| error::ServiceError::TestServerReadySignal)?;

    Ok(TestHarness { local_addr, server })
}

async fn serve(
    listener: TcpListener,
    state: Arc<state::AppState>,
    rpc_timeout: Option<std::time::Duration>,
    ready_tx: Option<oneshot::Sender<()>>,
) -> Result<(), error::ServiceError> {
    let music_service = servers::music::MusicService {
        state: state.clone(),
    };
    let cipher_service = servers::cipher::CipherService {
        state: state.clone(),
    };

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ServiceStatusServer<servers::status::StatusService>>()
        .await;
    health_reporter
        .set_serving::<YtMusicServer<servers::music::MusicService>>()
        .await;
    health_reporter
        .set_serving::<YtCipherServer<servers::cipher::CipherService>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()
        .map_err(error::ServiceError::Reflection)?;

    let incoming = tonic::transport::server::TcpIncoming::from_listener(listener, true, None)
        .map_err(error::ServiceError::Incoming)?;
    let mut server = tonic::transport::Server::builder();
    if let Some(rpc_timeout) = rpc_timeout {
        server = server.timeout(rpc_timeout);
    }
    let server = server
        .add_service(health_service)
        .add_service(reflection)
        .add_service(YtMusicServer::new(music_service))
        .add_service(YtCipherServer::new(cipher_service))
        .add_service(ServiceStatusServer::new(servers::status::StatusService {
            state,
        }))
        .serve_with_incoming(incoming);

    if let Some(ready_tx) = ready_tx {
        let _ = ready_tx.send(());
    }

    server.await.map_err(error::ServiceError::Transport)
}

async fn bind_service_listener(
    listen_addr: SocketAddr,
) -> Result<TcpListener, error::ServiceError> {
    TcpListener::bind(listen_addr)
        .await
        .map_err(error::ServiceError::ListenerBind)
}

#[doc(hidden)]
pub struct TestHarness {
    local_addr: SocketAddr,
    server: JoinHandle<Result<(), error::ServiceError>>,
}

impl TestHarness {
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        self.server.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::bind_service_listener;

    #[tokio::test]
    async fn bind_service_listener_binds_an_ephemeral_port() {
        let listener = bind_service_listener("127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();

        assert_ne!(listener.local_addr().unwrap().port(), 0);
    }
}
