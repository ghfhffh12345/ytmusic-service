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

    let (mut reporter, health) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<
            proto::ytmusic::v1::yt_music_public_server::YtMusicPublicServer<
                servers::public::PublicService,
            >,
        >()
        .await;
    reporter
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

    let public = tonic::transport::Server::builder()
        .add_service(health.clone())
        .add_service(
            proto::ytmusic::v1::yt_music_public_server::YtMusicPublicServer::new(public_service),
        )
        .serve(config.public_addr());

    let admin = tonic::transport::Server::builder()
        .add_service(health)
        .add_service(reflection)
        .add_service(
            proto::ytmusic::v1::admin::yt_music_admin_server::YtMusicAdminServer::new(
                admin_service,
            ),
        )
        .serve(config.admin_addr());

    tokio::try_join!(public, admin).map_err(error::ServiceError::Transport)?;
    Ok(())
}
