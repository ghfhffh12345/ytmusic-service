use tempfile::NamedTempFile;
use tonic::transport::Channel;
use ytmusic_service::config::ServiceConfig;
use ytmusic_service::state::SharedCipher;
use ytmusic_service_proto::ytmusic::v2::{
    self as pb, service_status_client::ServiceStatusClient, yt_cipher_client::YtCipherClient,
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

async fn cipher_client(
    harness: &ytmusic_service::TestHarness,
) -> Result<YtCipherClient<Channel>, Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", harness.local_addr());
    let channel = Channel::from_shared(endpoint)?.connect().await?;
    Ok(YtCipherClient::new(channel))
}

async fn status_client(
    harness: &ytmusic_service::TestHarness,
) -> Result<ServiceStatusClient<Channel>, Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", harness.local_addr());
    let channel = Channel::from_shared(endpoint)?.connect().await?;
    Ok(ServiceStatusClient::new(channel))
}

async fn test_harness_with_cipher_timestamp(
    signature_timestamp: u32,
) -> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .browser_auth_path(browser_json.into_temp_path().keep()?)
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        SharedCipher::fixed_signature_timestamp_for_tests(signature_timestamp),
    )
    .await?)
}

async fn test_harness_with_minimal_state()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .browser_auth_path(browser_json.into_temp_path().keep()?)
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        SharedCipher::fixed_signature_timestamp_for_tests(20_577),
    )
    .await?)
}

async fn test_harness_with_broken_cipher()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .browser_auth_path(browser_json.into_temp_path().keep()?)
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        SharedCipher::failing_refresh_for_tests(yt_cipher::Error::SignatureDecipherFailed),
    )
    .await?)
}

#[tokio::test]
async fn get_signature_timestamp_returns_current_value() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_cipher_timestamp(20_577).await?;
    let mut client = cipher_client(&harness).await?;

    let response = client
        .get_signature_timestamp(pb::Empty {})
        .await?
        .into_inner();

    assert_eq!(response.signature_timestamp, 20_577);

    Ok(())
}

#[tokio::test]
async fn decipher_rejects_blank_signature_cipher() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_minimal_state().await?;
    let mut client = cipher_client(&harness).await?;

    let error = client
        .decipher(pb::DecipherRequest {
            signature_cipher: "  ".to_owned(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), tonic::Code::InvalidArgument);

    Ok(())
}

#[tokio::test]
async fn subsystem_metadata_is_present_on_upstream_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_broken_cipher().await?;
    let mut client = cipher_client(&harness).await?;

    let error = client.refresh(pb::Empty {}).await.unwrap_err();

    assert_eq!(error.code(), tonic::Code::Internal);
    assert_eq!(
        error.metadata().get("ytmusic-service-subsystem").unwrap(),
        "cipher"
    );

    Ok(())
}

#[tokio::test]
async fn status_reports_cipher_and_ytmusic_ready() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_minimal_state().await?;
    let mut client = status_client(&harness).await?;

    let response = client
        .get_status(pb::GetStatusRequest {})
        .await?
        .into_inner();

    assert!(response.ytmusic_ready);
    assert!(response.cipher_ready);

    Ok(())
}
