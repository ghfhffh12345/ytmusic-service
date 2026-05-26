use std::{fs, path::PathBuf};

use tempfile::NamedTempFile;
use tonic::transport::Channel;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};
use ytmusic_service::config::ServiceConfig;
use ytmusic_service_proto::ytmusic::v2::{
    self as pb, search_result, yt_music_client::YtMusicClient,
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

fn ytmusicapi_fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(std::env::var("HOME").expect("HOME is set"))
        .join(".cargo/git/checkouts/ytmusicapi-5783a8c22a58856f/7827779/crates/ytmusicapi/tests")
        .join(relative)
}

fn read_ytmusicapi_fixture(relative: &str) -> String {
    fs::read_to_string(ytmusicapi_fixture_path(relative)).expect("fixture is readable")
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

async fn test_harness_with_mocked_music_responses()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/search"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(read_ytmusicapi_fixture(
                "fixtures/search/raw/songs_authenticated.json",
            )),
        )
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::unavailable_for_tests(),
    )
    .await?)
}

async fn test_harness_with_minimal_state()
-> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;
    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::unavailable_for_tests(),
    )
    .await?)
}

async fn test_harness_with_song_response_and_signature_timestamp(
    signature_timestamp: u32,
) -> Result<ytmusic_service::TestHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/player"))
        .and(move |request: &Request| {
            let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
            body["playbackContext"]["contentPlaybackContext"]["signatureTimestamp"]
                == serde_json::Value::from(signature_timestamp)
        })
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_ytmusicapi_fixture("fixtures/song/raw/response1.json")),
        )
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .build()?;

    Ok(ytmusic_service::run_for_tests_with_parts(
        config,
        music,
        ytmusic_service::state::SharedCipher::fixed_signature_timestamp_for_tests(
            signature_timestamp,
        ),
    )
    .await?)
}

#[tokio::test]
async fn search_maps_results_into_v2_oneof_variants() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_mocked_music_responses().await?;
    let mut client = grpc_client(&harness).await?;

    let response = client
        .search(pb::SearchRequest {
            query: "miles davis".to_owned(),
            filter: Some(pb::SearchFilter::Songs as i32),
            ignore_spelling: false,
        })
        .await?
        .into_inner();

    assert!(!response.items.is_empty());
    assert!(matches!(
        response.items[0].kind,
        Some(search_result::Kind::Song(_))
    ));

    Ok(())
}

#[tokio::test]
async fn get_watch_playlist_rejects_missing_video_and_playlist_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_minimal_state().await?;
    let mut client = grpc_client(&harness).await?;
    let error = client
        .get_watch_playlist(pb::GetWatchPlaylistRequest {
            video_id: None,
            playlist_id: None,
            radio: false,
            shuffle: false,
        })
        .await
        .unwrap_err();

    assert_eq!(error.code(), tonic::Code::InvalidArgument);
    Ok(())
}

#[tokio::test]
async fn get_song_uses_cipher_signature_timestamp_and_maps_streaming_data()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_song_response_and_signature_timestamp(20_577).await?;
    let mut client = grpc_client(&harness).await?;

    let response = client
        .get_song(pb::GetSongRequest {
            video_id: "0rilIYWiJ7M".to_owned(),
        })
        .await?
        .into_inner();

    assert_eq!(
        response.video_details.as_ref().unwrap().video_id,
        "0rilIYWiJ7M"
    );
    assert!(!response.streaming_data.as_ref().unwrap().formats.is_empty());

    Ok(())
}
