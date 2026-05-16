use tempfile::TempDir;
use ytmusic_service::error::ServiceError;

#[cfg(unix)]
use std::os::unix::fs::symlink;

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
    std::fs::write(&path, "{}").unwrap();

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

    assert!(result.is_err());
}
