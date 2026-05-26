use std::{fs, path::PathBuf};

use serde_json::json;
use tempfile::NamedTempFile;
use tonic::transport::Channel;
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use ytmusic_service::config::ServiceConfig;
use ytmusic_service_proto::ytmusic::v2::{self as pb, yt_music_client::YtMusicClient};

struct MockedHarness {
    _server: MockServer,
    harness: ytmusic_service::TestHarness,
}

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

impl MockedHarness {
    async fn music_client(&self) -> Result<YtMusicClient<Channel>, Box<dyn std::error::Error>> {
        grpc_client(&self.harness).await
    }
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
-> Result<MockedHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/account/account_menu"))
        .and(query_param("alt", "json"))
        .and(query_param("key", "test-api-key"))
        .and(body_json(json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20250501.03.00",
                }
            }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_fixture("account/raw/account_info.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(MockedHarness {
        _server: server,
        harness: ytmusic_service::run_for_tests_with_parts(
            config,
            music,
            ytmusic_service::state::SharedCipher::unavailable_for_tests(),
        )
        .await?,
    })
}

async fn test_harness_with_liked_songs_response()
-> Result<MockedHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/browse"))
        .and(query_param("alt", "json"))
        .and(query_param("key", "test-api-key"))
        .and(body_json(json!({
            "browseId": "VLLM",
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20250501.03.00",
                }
            }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_fixture("liked_songs/raw/response1.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(MockedHarness {
        _server: server,
        harness: ytmusic_service::run_for_tests_with_parts(
            config,
            music,
            ytmusic_service::state::SharedCipher::unavailable_for_tests(),
        )
        .await?,
    })
}

async fn test_harness_with_library_playlists_response()
-> Result<MockedHarness, Box<dyn std::error::Error>> {
    let server = mocked_music_server().await;

    Mock::given(method("POST"))
        .and(path("/youtubei/v1/browse"))
        .and(query_param("alt", "json"))
        .and(query_param("key", "test-api-key"))
        .and(body_json(json!({
            "browseId": "FEmusic_liked_playlists",
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20250501.03.00",
                }
            }
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(read_fixture("library_playlists/raw/response1.json")),
        )
        .expect(1)
        .mount(&server)
        .await;

    let browser_json = test_browser_auth_file()?;
    let config = ServiceConfig::from_parts("127.0.0.1:0", browser_json.path())?;
    let music = ytmusicapi::YtMusic::builder()
        .homepage_url(server.uri())
        .base_url(format!("{}/youtubei/v1/", server.uri()))
        .browser_auth_path(browser_json.path())
        .build()?;

    Ok(MockedHarness {
        _server: server,
        harness: ytmusic_service::run_for_tests_with_parts(
            config,
            music,
            ytmusic_service::state::SharedCipher::unavailable_for_tests(),
        )
        .await?,
    })
}

#[tokio::test]
async fn get_account_info_maps_account_fields() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_account_info_response().await?;
    let mut client = harness.music_client().await?;

    let response = client.get_account_info(pb::Empty {}).await?.into_inner();

    assert_eq!(response.account_name, "Test Account");
    assert_eq!(response.channel_handle.as_deref(), Some("@test"));
    assert_eq!(
        response.account_photo_url,
        "https://example.com/account.jpg"
    );

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
async fn get_library_playlists_maps_playlist_fields_and_continuation_token()
-> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_library_playlists_response().await?;
    let mut client = harness.music_client().await?;

    let response = client
        .get_library_playlists(pb::Empty {})
        .await?
        .into_inner();

    assert_eq!(
        response.continuation_token.as_deref(),
        Some("playlist-token-1")
    );
    assert_eq!(response.items.len(), 2);

    let first = &response.items[0];
    assert_eq!(first.playlist_id, "PL123");
    assert_eq!(first.title.as_deref(), Some("Synthwave Mix"));
    assert_eq!(first.authors.len(), 1);
    assert_eq!(first.authors[0].name, "OpenAI");
    assert_eq!(first.item_count, Some(15));
    assert_eq!(first.thumbnails.len(), 1);
    assert_eq!(first.thumbnails[0].url, "https://example.com/1.jpg");

    let second = &response.items[1];
    assert_eq!(second.playlist_id, "PL999");
    assert_eq!(second.title, None);
    assert_eq!(second.authors.len(), 1);
    assert_eq!(second.authors[0].name, "Archive");
    assert_eq!(second.item_count, None);

    Ok(())
}

#[tokio::test]
async fn liked_songs_maps_items_and_continuation_token() -> Result<(), Box<dyn std::error::Error>> {
    let harness = test_harness_with_liked_songs_response().await?;
    let mut client = harness.music_client().await?;

    let response = client.get_liked_songs(pb::Empty {}).await?.into_inner();

    assert_eq!(response.playlist_id, "LM");
    assert_eq!(response.title, "Liked songs");
    assert_eq!(
        response.continuation_token.as_deref(),
        Some("liked-token-1")
    );
    assert_eq!(response.thumbnails.len(), 1);
    assert_eq!(
        response.thumbnails[0].url,
        "https://example.com/liked-songs.jpg"
    );
    assert_eq!(response.items.len(), 1);

    let first = &response.items[0];
    assert_eq!(first.video_id, "liked-song-1");
    assert_eq!(first.title, "Roygbiv");
    assert_eq!(first.artists.len(), 1);
    assert_eq!(first.artists[0].id, "UCBOC");
    assert_eq!(first.artists[0].name, "Boards of Canada");
    assert_eq!(first.album.as_ref().unwrap().id, "MPREb_album_1");
    assert_eq!(
        first.album.as_ref().unwrap().name,
        "Music Has the Right to Children"
    );
    assert_eq!(first.duration.as_deref(), Some("2:31"));
    assert_eq!(first.thumbnails.len(), 1);
    assert_eq!(
        first.thumbnails[0].url,
        "https://example.com/liked-song-1.jpg"
    );
    assert_eq!(first.like_status, Some(pb::LibraryLikeStatus::Like as i32));

    Ok(())
}
