use tempfile::TempDir;
use tokio::sync::Mutex;
use ytmusic_service::error::ServiceError;

#[cfg(unix)]
use std::os::unix::fs::symlink;

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

fn is_lower_hex_token(value: &str) -> bool {
    value
        .chars()
        .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
}

#[tokio::test]
async fn startup_fails_when_browser_json_is_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");

    let result = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path.clone(),
    );

    match result {
        Err(ServiceError::BrowserAuthPathMissing(returned_path)) => {
            assert_eq!(returned_path, path);
        }
        other => panic!("expected BrowserAuthPathMissing, got {other:?}"),
    }
}

#[tokio::test]
async fn startup_fails_when_browser_json_path_is_a_directory() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::create_dir(&path).unwrap();

    let result = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path.clone(),
    );

    match result {
        Err(ServiceError::BrowserAuthPathNotFile(returned_path)) => {
            assert_eq!(returned_path, path);
        }
        other => panic!("expected BrowserAuthPathNotFile, got {other:?}"),
    }
}

#[tokio::test]
async fn startup_fails_when_public_addr_is_invalid() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, "{}").unwrap();

    let result =
        ytmusic_service::config::ServiceConfig::from_parts("invalid", "127.0.0.1:50052", path);

    assert!(matches!(result, Err(ServiceError::InvalidSocketAddress(_))));
}

#[cfg(unix)]
#[tokio::test]
async fn startup_accepts_browser_json_symlink_to_regular_file() {
    let dir = TempDir::new().unwrap();
    let target_path = dir.path().join("real-browser.json");
    std::fs::write(&target_path, "{}").unwrap();

    let symlink_path = dir.path().join("browser.json");
    symlink(&target_path, &symlink_path).unwrap();

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        symlink_path.clone(),
    )
    .unwrap();

    assert_eq!(config.browser_auth_path(), symlink_path.as_path());
}

#[tokio::test]
async fn startup_accepts_existing_browser_json_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    write_minimal_valid_browser_auth(&path);

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path.clone(),
    )
    .unwrap();

    assert_eq!(config.public_addr().to_string(), "127.0.0.1:50051");
    assert_eq!(config.admin_addr().to_string(), "127.0.0.1:50052");
    assert_eq!(config.browser_auth_path(), path.as_path());
}

#[tokio::test]
async fn startup_requires_valid_browser_auth_json() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, r#"{"cookie":"missing required auth headers"}"#).unwrap();

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    )
    .unwrap();

    let result = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config).await;

    assert!(matches!(result, Err(ServiceError::BrowserAuthLoad(_))));
}

#[tokio::test]
async fn startup_fails_when_browser_json_is_malformed() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, "{not-json").unwrap();

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    )
    .unwrap();

    let result = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config).await;

    assert!(matches!(result, Err(ServiceError::BrowserAuthLoad(_))));
}

#[tokio::test]
async fn startup_fails_when_browser_json_probe_fails() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    write_minimal_valid_browser_auth(&path);

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    )
    .unwrap();
    let seen = std::sync::Arc::new(Mutex::new(Vec::new()));
    let validator: ytmusic_service::StartupAuthValidator = std::sync::Arc::new({
        let seen = std::sync::Arc::clone(&seen);
        move |candidate| {
            let seen = std::sync::Arc::clone(&seen);
            let candidate_version = candidate.version.to_string();
            Box::pin(async move {
                seen.lock().await.push(candidate_version);
                Err(ServiceError::YtMusic(ytmusicapi::Error::AuthValidation(
                    "probe rejected auth".to_owned(),
                )))
            })
        }
    });

    let result = ytmusic_service::load_startup_auth_for_tests(&config, validator).await;

    assert!(matches!(result, Err(ServiceError::YtMusic(_))));
    assert_eq!(seen.lock().await.len(), 1);
}

#[tokio::test]
async fn startup_assigns_unique_auth_context_versions_per_successful_load() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    write_minimal_valid_browser_auth(&path);
    let path_display = path.display().to_string();

    let config = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    )
    .unwrap();

    let first = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config)
        .await
        .unwrap();
    let second = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config)
        .await
        .unwrap();

    assert_ne!(first.version.as_ref(), second.version.as_ref());
    assert_eq!(first.version.len(), 32);
    assert_eq!(second.version.len(), 32);
    assert!(is_lower_hex_token(first.version.as_ref()));
    assert!(is_lower_hex_token(second.version.as_ref()));
    assert!(!first.version.contains(&path_display));
    assert!(!second.version.contains(&path_display));
}
