use std::sync::Arc;
use std::time::SystemTime;

use arc_swap::ArcSwap;
use tempfile::TempDir;
use ytmusic_service::{
    auth_context::AuthContext,
    config::ServiceConfig,
    error::ServiceError,
    state::{AppState, SharedCipher},
};

#[test]
fn reload_keeps_previous_context_on_validation_failure() {
    let initial = Arc::new("v1".to_owned());
    let swap = ArcSwap::from(initial.clone());

    let loaded = swap.load();
    assert_eq!(loaded.as_str(), "v1");
}

fn valid_browser_auth_json() -> &'static str {
    r#"{
  "cookie": "__Secure-3PAPISID=test-sapisid",
  "x-goog-authuser": "0"
}"#
}

fn test_auth_context(version: &'static str) -> AuthContext {
    AuthContext {
        client: ytmusicapi::YtMusic::new().expect("test client"),
        version: Arc::<str>::from(version),
        loaded_at: SystemTime::UNIX_EPOCH,
    }
}

fn test_state(version: &'static str) -> AppState {
    AppState::from_parts_for_tests(
        test_auth_context(version),
        Arc::new(SharedCipher::unavailable_for_tests()),
    )
}

#[tokio::test]
async fn reload_swaps_context_after_successful_probe() {
    let state = test_state("v1");
    let old_version = state.auth.load().version.to_string();

    let new_version = state
        .activate_auth_context_for_tests(test_auth_context("v2"), |_| async { Ok(()) })
        .await
        .unwrap();

    assert_ne!(old_version, new_version);
    assert_eq!(state.auth.load().version.as_ref(), "v2");
}

#[tokio::test]
async fn reload_failure_preserves_prior_version() {
    let state = test_state("v1");
    let old_version = state.auth.load().version.to_string();

    let result = state
        .activate_auth_context_for_tests(test_auth_context("v2"), |_| async {
            Err(ServiceError::YtMusic(ytmusicapi::Error::AuthValidation(
                "probe rejected auth".to_owned(),
            )))
        })
        .await;

    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}

#[tokio::test]
async fn reload_browser_auth_preserves_prior_version_when_load_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let initial = AuthContext::from_browser_auth_file(&config).await.unwrap();
    let state =
        AppState::from_parts_for_tests(initial, Arc::new(SharedCipher::unavailable_for_tests()));
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, r#"{"cookie":"broken"}"#).unwrap();

    let result = state.reload_browser_auth(&config).await;

    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}
