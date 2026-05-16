use std::sync::Arc;

use tempfile::TempDir;
use ytmusic_service::{
    auth_context::AuthContext,
    config::ServiceConfig,
    error::ServiceError,
    state::{AppState, SharedCipher},
};

fn valid_browser_auth_json() -> &'static str {
    r#"{
  "cookie": "__Secure-3PAPISID=test-sapisid",
  "x-goog-authuser": "0"
}"#
}

fn probe_ok(_: &AuthContext) -> Result<(), ServiceError> {
    Ok(())
}

fn probe_validation_failure(_: &AuthContext) -> Result<(), ServiceError> {
    Err(ServiceError::YtMusic(ytmusicapi::Error::AuthValidation(
        "probe rejected auth".to_owned(),
    )))
}

async fn test_state_with_probe(
    config: &ServiceConfig,
    probe: fn(&AuthContext) -> Result<(), ServiceError>,
) -> AppState {
    let initial = AuthContext::from_browser_auth_file(config).await.unwrap();

    AppState::from_parts_for_reload_tests(
        initial,
        Arc::new(SharedCipher::unavailable_for_tests()),
        Arc::new(probe),
    )
}

#[tokio::test]
async fn reload_keeps_previous_context_on_validation_failure() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let state = test_state_with_probe(&config, probe_validation_failure).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let result = state.reload_browser_auth(&config).await;

    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}

#[tokio::test]
async fn reload_swaps_context_after_successful_probe() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let state = test_state_with_probe(&config, probe_ok).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let new_version = state.reload_browser_auth(&config).await.unwrap();

    assert_ne!(old_version, new_version);
    assert_eq!(state.auth.load().version.as_ref(), new_version);
}

#[tokio::test]
async fn reload_browser_auth_preserves_prior_version_when_load_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let state = test_state_with_probe(&config, probe_ok).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, r#"{"cookie":"broken"}"#).unwrap();

    let result = state.reload_browser_auth(&config).await;

    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}
