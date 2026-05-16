use std::sync::Arc;

use tempfile::TempDir;
use tokio::sync::{Mutex, Notify};
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

type Probe = Arc<dyn for<'a> Fn(&'a AuthContext) -> ProbeFuture<'a> + Send + Sync>;
type ProbeFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ServiceError>> + Send + 'a>>;

fn ok_probe() -> Probe {
    Arc::new(|_| Box::pin(async { Ok(()) }))
}

fn validation_failure_probe(seen: Arc<Mutex<Vec<String>>>) -> Probe {
    Arc::new(move |candidate| {
        let seen = Arc::clone(&seen);
        let candidate_version = candidate.version.to_string();
        Box::pin(async move {
            seen.lock().await.push(candidate_version);
            Err(ServiceError::YtMusic(ytmusicapi::Error::AuthValidation(
                "probe rejected auth".to_owned(),
            )))
        })
    })
}

async fn test_state_with_probe(config: &ServiceConfig, probe: Probe) -> AppState {
    let initial = AuthContext::from_browser_auth_file(config).await.unwrap();

    AppState::from_parts_for_reload_tests(
        initial,
        Arc::new(SharedCipher::unavailable_for_tests()),
        probe,
    )
}

#[tokio::test]
async fn reload_keeps_previous_context_on_validation_failure() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let state = test_state_with_probe(&config, validation_failure_probe(Arc::clone(&seen))).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let result = state.reload_browser_auth(&config).await;

    assert!(result.is_err());
    let seen = seen.lock().await;
    assert_eq!(seen.len(), 1);
    assert_ne!(seen[0], old_version);
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}

#[tokio::test]
async fn reload_swaps_context_after_successful_probe() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let probe: Probe = Arc::new({
        let seen = Arc::clone(&seen);
        move |candidate| {
            let seen = Arc::clone(&seen);
            let candidate_version = candidate.version.to_string();
            Box::pin(async move {
                seen.lock().await.push(candidate_version);
                Ok(())
            })
        }
    });
    let state = test_state_with_probe(&config, probe).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let new_version = state.reload_browser_auth(&config).await.unwrap();

    let seen = seen.lock().await;
    assert_eq!(seen.as_slice(), &[new_version.clone()]);
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
    let state = test_state_with_probe(&config, ok_probe()).await;
    let old_version = state.auth.load().version.to_string();

    std::fs::write(&path, r#"{"cookie":"broken"}"#).unwrap();

    let result = state.reload_browser_auth(&config).await;

    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}

#[tokio::test]
async fn concurrent_reloads_are_serialized() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, valid_browser_auth_json()).unwrap();

    let config =
        ServiceConfig::from_parts("127.0.0.1:50051", "127.0.0.1:50052", path.clone()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let first_started = Arc::new(Notify::new());
    let release_first = Arc::new(Notify::new());
    let first_completed = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let state = Arc::new(
        test_state_with_probe(
            &config,
            Arc::new({
                let seen = Arc::clone(&seen);
                let first_started = Arc::clone(&first_started);
                let release_first = Arc::clone(&release_first);
                let first_completed = Arc::clone(&first_completed);
                move |candidate| {
                    let seen = Arc::clone(&seen);
                    let first_started = Arc::clone(&first_started);
                    let release_first = Arc::clone(&release_first);
                    let first_completed = Arc::clone(&first_completed);
                    let candidate_version = candidate.version.to_string();
                    Box::pin(async move {
                        let mut seen_guard = seen.lock().await;
                        seen_guard.push(candidate_version);
                        let call_index = seen_guard.len();
                        drop(seen_guard);

                        if call_index == 1 {
                            first_started.notify_one();
                            release_first.notified().await;
                            first_completed.store(true, std::sync::atomic::Ordering::SeqCst);
                        } else {
                            assert!(
                                first_completed.load(std::sync::atomic::Ordering::SeqCst),
                                "second validation started before first completed"
                            );
                        }

                        Ok(())
                    })
                }
            }),
        )
        .await,
    );

    let first_state = Arc::clone(&state);
    let first_config = config.clone();
    let first_reload = tokio::spawn(async move {
        first_state
            .reload_browser_auth(&first_config)
            .await
            .unwrap()
    });

    first_started.notified().await;

    let second_state = Arc::clone(&state);
    let second_config = config.clone();
    let second_reload = tokio::spawn(async move {
        second_state
            .reload_browser_auth(&second_config)
            .await
            .unwrap()
    });

    tokio::task::yield_now().await;
    release_first.notify_one();

    let first_version = first_reload.await.unwrap();
    let second_version = second_reload.await.unwrap();
    let seen = seen.lock().await;

    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0], first_version);
    assert_eq!(seen[1], second_version);
    assert_eq!(state.auth.load().version.as_ref(), second_version);
}
