use tempfile::NamedTempFile;
use ytmusic_service::config::ServiceConfig;
use ytmusic_service::state::SharedCipher;

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

#[tokio::test]
async fn rust_client_connects_to_in_process_server_and_calls_status()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_minimal_state().await?;
    let endpoint = format!("http://{}", harness.local_addr());
    let mut client = ytmusic_service_client::YtMusicServiceClient::connect(endpoint).await?;

    let status = client.status().get_status().await?;
    assert_eq!(status.lifecycle, "serving");

    Ok(())
}

#[test]
fn client_error_helpers_classify_status_codes() {
    let status = tonic::Status::unavailable("upstream unavailable");
    let error = ytmusic_service_client::error::ClientStatus::from(status);

    assert!(error.is_unavailable());
    assert!(!error.is_invalid_argument());
}
