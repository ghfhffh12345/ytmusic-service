use std::{fs, path::PathBuf};

use tempfile::NamedTempFile;
use tonic::transport::Channel;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use ytmusic_service::config::ServiceConfig;
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_music_client::YtMusicClient};

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

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

fn read_fixture(relative: &str) -> String {
    fs::read_to_string(fixture_path(relative)).expect("fixture is readable")
}

async fn mocked_music_server() -> MockServer {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"ytcfg.set({ "VISITOR_DATA": "visitor-id-123", "INNERTUBE_API_KEY": "test-api-key", "INNERTUBE_CONTEXT_CLIENT_VERSION": "1.20250501.03.00" });"#,
        ))
        .mount(&server)
        .await;

    server
}

async fn grpc_client(
    harness: &ytmusic_service::TestHarness,
) -> Result<YtMusicClient<Channel>, Box<dyn std::error::Error>> {
    let endpoint = format!("http://{}", harness.local_addr());
    let channel = Channel::from_shared(endpoint)?.connect().await?;
    Ok(YtMusicClient::new(channel))
}

async fn test_harness_with_minimal_state()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;
    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::unavailable_for_tests(),
    )
    .await?)
}

async fn test_harness_with_account_info_response()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/account/account_menu"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_fixture("account/raw/account_info.json")),
        )
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::unavailable_for_tests(),
    )
    .await?)
}

async fn test_harness_with_liked_songs_response()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/browse"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_fixture("liked_songs/raw/response1.json")),
        )
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::unavailable_for_tests(),
    )
    .await?)
}

#[tokio::test]
async fn get_account_info_maps_account_fields() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_account_info_response().await?;
    let mut client = grpc_client(&harness).await?;

    let response = client.get_account_info(pb::Empty {}).await?.into_inner();

    assert_eq!(response.account_name, "Test Account");
    assert_eq!(response.channel_handle.as_deref(), Some("@test"));

    Ok(())
}

#[tokio::test]
async fn library_playlists_continuation_rejects_blank_tokens()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_minimal_state().await?;
    let mut client = grpc_client(&harness).await?;

    let error = client
        .get_library_playlists_continuation(pb::ContinuationRequest {
            token: "   ".to_owned(),
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), tonic::Code::InvalidArgument);

    Ok(())
}

#[tokio::test]
async fn liked_songs_maps_items_and_continuation_token() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_liked_songs_response().await?;
    let mut client = grpc_client(&harness).await?;

    let response = client.get_liked_songs(pb::Empty {}).await?.into_inner();

    assert_eq!(response.title, "Liked songs");
    assert!(!response.items.is_empty());
    assert!(response.continuation_token.is_some());

    Ok(())
}
