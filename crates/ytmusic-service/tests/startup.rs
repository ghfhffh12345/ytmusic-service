use tempfile::NamedTempFile;
use tonic::codegen::tokio_stream::StreamExt as _;
use tonic::{Code, Request, transport::Channel};
use tonic_reflection::pb::v1::{
    ServerReflectionRequest, server_reflection_client::ServerReflectionClient,
    server_reflection_request::MessageRequest, server_reflection_response::MessageResponse,
};

fn write_minimal_valid_browser_auth(path: &std::path::Path) {
    std::fs::write(
        path,
        r#"{
  "cookie": "__Secure-3PAPISID=test-sapisid",
  "x-goog-authuser": "0"
}"#,
    )
    .unwrap();
}

fn test_browser_auth_file() -> std::io::Result<NamedTempFile> {
    let browser_json = NamedTempFile::new()?;
    write_minimal_valid_browser_auth(browser_json.path());
    Ok(browser_json)
}

#[tokio::test]
async fn startup_serves_health_and_status_on_one_listener() -> Result<(), Box<dyn std::error::Error>>
{
    let browser_json = test_browser_auth_file()?;
    let config =
        ytmusic_service::config::ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;

    let harness = ytmusic_service::run_for_tests(config).await?;
    let endpoint = format!("http://{}", harness.local_addr());
    let channel = Channel::from_shared(endpoint.clone())?.connect().await?;

    let mut health = tonic_health::pb::health_client::HealthClient::new(channel.clone());
    let health_response = health
        .check(tonic_health::pb::HealthCheckRequest {
            service: "ytmusic.v2.ServiceStatus".to_owned(),
        })
        .await?
        .into_inner();

    assert_eq!(
        health_response.status,
        tonic_health::pb::health_check_response::ServingStatus::Serving as i32
    );

    let mut status =
        ytmusic_service_proto::ytmusic::v2::service_status_client::ServiceStatusClient::new(
            channel.clone(),
        );
    let status_response = status
        .get_status(ytmusic_service_proto::ytmusic::v2::GetStatusRequest {})
        .await?
        .into_inner();

    assert!(status_response.ytmusic_ready);
    assert!(status_response.cipher_ready);
    assert_eq!(status_response.lifecycle, "serving");

    let mut reflection = ServerReflectionClient::new(channel.clone());
    let reflection_request = tonic::codegen::tokio_stream::iter(vec![ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::ListServices(String::new())),
    }]);
    let mut reflection_responses = reflection
        .server_reflection_info(Request::new(reflection_request))
        .await?
        .into_inner();
    let reflection_response = reflection_responses
        .next()
        .await
        .expect("reflection response")
        .expect("successful reflection response");
    let services = match reflection_response
        .message_response
        .expect("reflection message response")
    {
        MessageResponse::ListServicesResponse(services) => services,
        other => panic!("unexpected reflection response: {other:?}"),
    };
    let service_names: Vec<_> = services
        .service
        .into_iter()
        .map(|service| service.name)
        .collect();
    assert!(
        service_names
            .iter()
            .any(|name| name == "grpc.reflection.v1.ServerReflection")
    );
    assert!(
        service_names
            .iter()
            .any(|name| name == "ytmusic.v2.ServiceStatus")
    );
    assert!(
        service_names
            .iter()
            .any(|name| name == "ytmusic.v2.YtMusic")
    );
    assert!(
        service_names
            .iter()
            .any(|name| name == "ytmusic.v2.YtCipher")
    );

    let mut music =
        ytmusic_service_proto::ytmusic::v2::yt_music_client::YtMusicClient::new(channel.clone());
    let music_status = music
        .get_library_playlists(ytmusic_service_proto::ytmusic::v2::Empty {})
        .await
        .unwrap_err();
    assert_eq!(music_status.code(), Code::Unimplemented);
    assert!(music_status.message().contains("not implemented yet"));

    let mut cipher =
        ytmusic_service_proto::ytmusic::v2::yt_cipher_client::YtCipherClient::new(channel);
    let cipher_status = cipher
        .get_signature_timestamp(ytmusic_service_proto::ytmusic::v2::Empty {})
        .await
        .unwrap_err();
    assert_eq!(cipher_status.code(), Code::Unimplemented);
    assert_eq!(
        cipher_status.message(),
        "ytmusic.v2.YtCipher RPCs are not implemented yet"
    );

    Ok(())
}
