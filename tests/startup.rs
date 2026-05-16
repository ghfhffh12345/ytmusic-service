use tempfile::TempDir;

#[tokio::test]
async fn startup_fails_when_browser_json_is_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");

    let result = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    );

    assert!(result.is_err());
}

#[tokio::test]
async fn startup_accepts_existing_browser_json_path() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("browser.json");
    std::fs::write(&path, "{}").unwrap();

    let result = ytmusic_service::config::ServiceConfig::from_parts(
        "127.0.0.1:50051",
        "127.0.0.1:50052",
        path,
    );

    assert!(result.is_ok());
}
