use std::{sync::Arc, time::SystemTime};

use tonic::{Code, Request};
use ytmusic_service::{
    auth_context::AuthContext,
    error::{ServiceError, map_service_error},
    proto::ytmusic::v1::{
        AccountInfoResponse, Empty, GetLibraryPlaylistsContinuationRequest, SearchRequest,
        yt_music_public_server::YtMusicPublic,
    },
    servers::public::PublicService,
    state::{AppState, SharedCipher},
};

fn test_public_service() -> PublicService {
    let auth = AuthContext {
        client: ytmusicapi::YtMusic::new().expect("test client"),
        version: Arc::<str>::from("test-version"),
        loaded_at: SystemTime::UNIX_EPOCH,
    };

    PublicService {
        state: Arc::new(AppState::from_parts_for_tests(
            auth,
            Arc::new(SharedCipher::unavailable_for_tests()),
        )),
    }
}

#[tokio::test]
async fn public_search_rejects_empty_query() {
    let service = test_public_service();

    let status = service
        .search(Request::new(SearchRequest {
            query: "   ".to_owned(),
            filter: None,
            ignore_spelling: false,
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn public_api_library_playlists_surfaces_browser_auth_requirement() {
    let service = test_public_service();

    let status = service
        .get_library_playlists(Request::new(Empty {}))
        .await
        .unwrap_err();

    assert_eq!(status.code(), Code::Unimplemented);
    assert!(
        !status
            .message()
            .contains("adapter wiring has not been added yet"),
        "unexpected status message: {}",
        status.message()
    );
}

#[tokio::test]
async fn public_api_get_account_info_surfaces_runtime_status_not_stub_status() {
    let service = test_public_service();

    let status = service
        .get_account_info(Request::new(Empty {}))
        .await
        .unwrap_err();

    assert_eq!(status.code(), Code::Unimplemented);
    assert!(
        !status
            .message()
            .contains("adapter wiring has not been added yet"),
        "unexpected status message: {}",
        status.message()
    );
}

#[tokio::test]
async fn public_api_library_playlists_continuation_rejects_empty_token() {
    let service = test_public_service();

    let status = service
        .get_library_playlists_continuation(Request::new(GetLibraryPlaylistsContinuationRequest {
            token: "   ".to_owned(),
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(
        status.message(),
        "library playlists continuation token must not be empty"
    );
}

#[tokio::test]
async fn public_api_account_info_response_keeps_name_field() {
    let response = AccountInfoResponse {
        account_name: "listener@example.com".to_owned(),
        channel_handle: None,
        account_photo_url: String::new(),
    };

    assert_eq!(response.account_name, "listener@example.com");
}

#[test]
fn map_service_error_preserves_status_categories() {
    let unavailable = map_service_error(&ServiceError::CipherWorkerUnavailable);
    assert_eq!(unavailable.code(), Code::Unavailable);

    let invalid_argument = map_service_error(&ServiceError::YtMusic(
        ytmusicapi::Error::InvalidInput("query must not be blank".to_owned()),
    ));
    assert_eq!(invalid_argument.code(), Code::InvalidArgument);
}

#[test]
fn admin_placeholder_module_is_explicit() {
    assert_eq!(
        ytmusic_service::servers::admin::PLACEHOLDER_MESSAGE,
        "admin service placeholder"
    );
}
