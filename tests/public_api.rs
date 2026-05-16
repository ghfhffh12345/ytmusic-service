use std::{sync::Arc, time::SystemTime};

use tonic::{Code, Request};
use ytmusic_service::{
    auth_context::AuthContext,
    proto::ytmusic::v1::{SearchRequest, yt_music_public_server::YtMusicPublic},
    servers::public::PublicService,
    state::{AppState, SharedCipher},
};

#[tokio::test]
async fn public_search_rejects_empty_query() {
    let auth = AuthContext {
        client: ytmusicapi::YtMusic::new().expect("test client"),
        version: Arc::<str>::from("test-version"),
        loaded_at: SystemTime::UNIX_EPOCH,
    };
    let service = PublicService {
        state: Arc::new(AppState::from_parts(
            auth,
            Arc::new(SharedCipher::unavailable()),
        )),
    };

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

#[test]
fn admin_placeholder_module_is_explicit() {
    assert_eq!(
        ytmusic_service::servers::admin::PLACEHOLDER_MESSAGE,
        "admin service placeholder"
    );
}
