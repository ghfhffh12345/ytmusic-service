use tempfile::NamedTempFile;
use tonic::transport::Channel;

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
            channel,
        );
    let status_response = status
        .get_status(ytmusic_service_proto::ytmusic::v2::GetStatusRequest {})
        .await?
        .into_inner();

    assert!(status_response.ytmusic_ready);
    assert!(status_response.cipher_ready);
    assert_eq!(status_response.lifecycle, "serving");

    Ok(())
}
