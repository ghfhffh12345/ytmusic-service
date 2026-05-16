pub mod adapters;
pub mod auth_context;
pub mod config;
pub mod error;
pub mod proto;
pub mod servers;
pub mod state;

pub async fn run(config: config::ServiceConfig) -> Result<(), error::ServiceError> {
    let auth = auth_context::AuthContext::from_browser_auth_file(&config).await?;
    auth.probe().await?;

    let state = std::sync::Arc::new(state::AppState::new(auth).await?);

    let public_service = servers::public::PublicService {
        state: state.clone(),
    };
    let admin_service = servers::admin::AdminService {
        state: state.clone(),
        config: config.clone(),
    };

    let (public_listener, admin_listener) =
        bind_service_listeners(config.public_addr(), config.admin_addr()).await?;

    let (mut public_reporter, public_health) = tonic_health::server::health_reporter();
    public_reporter
        .set_serving::<
            proto::ytmusic::v1::yt_music_public_server::YtMusicPublicServer<
                servers::public::PublicService,
            >,
        >()
        .await;
    let (mut admin_reporter, admin_health) = tonic_health::server::health_reporter();
    admin_reporter
        .set_serving::<
            proto::ytmusic::v1::admin::yt_music_admin_server::YtMusicAdminServer<
                servers::admin::AdminService,
            >,
        >()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::ytmusic::v1::PUBLIC_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(
            proto::ytmusic::v1::admin::ADMIN_FILE_DESCRIPTOR_SET,
        )
        .build_v1()
        .map_err(error::ServiceError::Reflection)?;

    let public_incoming =
        tonic::transport::server::TcpIncoming::from_listener(public_listener, true, None)
            .map_err(error::ServiceError::Incoming)?;
    let public = tonic::transport::Server::builder()
        .add_service(public_health)
        .add_service(
            proto::ytmusic::v1::yt_music_public_server::YtMusicPublicServer::new(public_service),
        )
        .serve_with_incoming(public_incoming);

    let admin_incoming =
        tonic::transport::server::TcpIncoming::from_listener(admin_listener, true, None)
            .map_err(error::ServiceError::Incoming)?;
    let admin = tonic::transport::Server::builder()
        .add_service(admin_health)
        .add_service(reflection)
        .add_service(
            proto::ytmusic::v1::admin::yt_music_admin_server::YtMusicAdminServer::new(
                admin_service,
            ),
        )
        .serve_with_incoming(admin_incoming);

    tokio::try_join!(public, admin).map_err(error::ServiceError::Transport)?;
    Ok(())
}

async fn bind_service_listeners(
    public_addr: std::net::SocketAddr,
    admin_addr: std::net::SocketAddr,
) -> Result<(tokio::net::TcpListener, tokio::net::TcpListener), error::ServiceError> {
    let public_listener = tokio::net::TcpListener::bind(public_addr)
        .await
        .map_err(error::ServiceError::ListenerBind)?;
    let admin_listener = tokio::net::TcpListener::bind(admin_addr)
        .await
        .map_err(error::ServiceError::ListenerBind)?;
    Ok((public_listener, admin_listener))
}

#[cfg(test)]
mod tests {
    use super::bind_service_listeners;
    use crate::error::ServiceError;

    #[tokio::test]
    async fn bind_service_listeners_releases_first_listener_when_second_bind_fails() {
        let occupied_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("occupied listener bind");
        let occupied_addr = occupied_listener
            .local_addr()
            .expect("occupied listener local addr");

        let public_probe_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("public probe listener bind");
        let public_addr = public_probe_listener
            .local_addr()
            .expect("public probe listener local addr");
        drop(public_probe_listener);

        let result = bind_service_listeners(public_addr, occupied_addr).await;

        assert!(matches!(result, Err(ServiceError::ListenerBind(_))));

        tokio::net::TcpListener::bind(public_addr)
            .await
            .expect("public addr should be released after failed dual bind");
    }
}
