# ytmusic-service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `ytmusic-service` as a Rust + `tonic` gRPC microservice that wraps `ytmusicapi` one-to-one, exposes a separate `Decipher` RPC backed by `yt-cipher`, requires valid `browser.json` at startup, supports atomic admin-triggered auth reload via `ArcSwap`, and ships in a distroless container image.

**Architecture:** A single Rust binary runs two `tonic` listeners in one Tokio runtime: a public API listener and a separate admin listener. Shared authenticated state lives behind `ArcSwap<Arc<AuthContext>>`, while `YtCipher` is initialized once and shared independently for `GetSong` and `Decipher`.

**Tech Stack:** Rust, Tokio, tonic, prost, tonic-build, tonic-health, tonic-reflection, arc-swap, reqwest, serde, ytmusicapi `v0.1.0`, yt-cipher `v0.1.0`, Docker/Podman with a distroless runtime image

---

## Planned File Structure

- `Cargo.toml`
  - workspace package manifest for the service binary and build dependencies
- `build.rs`
  - protobuf compilation and descriptor set generation
- `proto/ytmusic/v1/public.proto`
  - public API messages and service definitions
- `proto/ytmusic/v1/admin.proto`
  - admin API messages and service definitions
- `src/main.rs`
  - process bootstrap, config loading, startup validation, listener startup, graceful shutdown
- `src/lib.rs`
  - module wiring for tests and binary reuse
- `src/config.rs`
  - CLI/env configuration and validation
- `src/error.rs`
  - service-local error model and gRPC mapping helpers
- `src/auth_context.rs`
  - authenticated `YtMusic` construction and probe validation
- `src/state.rs`
  - shared runtime state, `ArcSwap`, and `YtCipher` ownership
- `src/proto.rs`
  - generated protobuf module includes and descriptor constant
- `src/adapters/mod.rs`
  - adapter module exports
- `src/adapters/ytmusic.rs`
  - one-to-one wrapper around `ytmusicapi`
- `src/adapters/cipher.rs`
  - wrapper around `yt-cipher`
- `src/servers/mod.rs`
  - tonic service module exports
- `src/servers/public.rs`
  - public gRPC service implementation
- `src/servers/admin.rs`
  - admin gRPC service implementation
- `tests/startup.rs`
  - startup success/failure coverage for `browser.json`
- `tests/public_api.rs`
  - public API and error translation coverage
- `tests/reload.rs`
  - reload and `ArcSwap` concurrency coverage
- `Dockerfile`
  - multi-stage build ending in a distroless runtime image
- `.dockerignore`
  - keep build context small
- `README.md`
  - local run, container run, and admin reload instructions

### Task 1: Bootstrap the Rust Service Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `build.rs`
- Create: `src/lib.rs`
- Create: `src/main.rs`
- Test: `cargo check`

- [ ] **Step 1: Write the failing manifest and entrypoint skeleton**

```toml
[package]
name = "ytmusic-service"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[dependencies]
arc-swap = "1.7"
prost = "0.13"
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "http2", "charset"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal", "sync"] }
tonic = { version = "0.12", features = ["transport"] }
tonic-health = "0.12"
tonic-reflection = "0.12"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
yt-cipher = { git = "https://github.com/ghfhffh12345/yt-cipher.git", tag = "v0.1.0" }
ytmusicapi = { git = "https://github.com/ghfhffh12345/ytmusicapi.git", tag = "v0.1.0" }

[build-dependencies]
tonic-build = "0.12"

[dev-dependencies]
tempfile = "3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
// src/main.rs
fn main() {
    println!("ytmusic-service bootstrap");
}
```

- [ ] **Step 2: Run compile check to verify the baseline fails**

Run: `cargo check`
Expected: FAIL because `build.rs` and protobuf inputs do not exist yet.

- [ ] **Step 3: Add minimal crate wiring**

```rust
// src/lib.rs
pub mod adapters;
pub mod auth_context;
pub mod config;
pub mod error;
pub mod proto;
pub mod servers;
pub mod state;
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
```

```rust
// src/main.rs
fn main() {
    tracing_subscriber::fmt::init();
}
```

- [ ] **Step 4: Run compile check to verify the crate shape is valid**

Run: `cargo check`
Expected: FAIL because referenced modules do not exist yet, but Cargo manifest and build script load correctly.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml build.rs src/lib.rs src/main.rs
git commit -m "chore: bootstrap ytmusic-service crate"
```

### Task 2: Define Protobuf Contracts and Code Generation

**Files:**
- Create: `proto/ytmusic/v1/public.proto`
- Create: `proto/ytmusic/v1/admin.proto`
- Modify: `build.rs`
- Create: `src/proto.rs`
- Test: `cargo check`

- [ ] **Step 1: Write the failing protobuf definitions**

```proto
syntax = "proto3";

package ytmusic.v1;

service YtMusicPublic {
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc SearchContinuation(SearchContinuationRequest) returns (SearchResponse);
  rpc GetWatchPlaylist(GetWatchPlaylistRequest) returns (WatchPlaylistResponse);
  rpc GetWatchPlaylistContinuation(GetWatchPlaylistContinuationRequest) returns (WatchPlaylistResponse);
  rpc GetSong(GetSongRequest) returns (GetSongResponse);
  rpc GetLibraryPlaylists(Empty) returns (LibraryPlaylistsResponse);
  rpc GetLibraryPlaylistsContinuation(LibraryContinuationRequest) returns (LibraryPlaylistsResponse);
  rpc GetAccountInfo(Empty) returns (AccountInfoResponse);
  rpc GetLibraryArtists(Empty) returns (LibraryArtistsResponse);
  rpc GetLibraryArtistsContinuation(LibraryContinuationRequest) returns (LibraryArtistsResponse);
  rpc GetLibraryAlbums(Empty) returns (LibraryAlbumsResponse);
  rpc GetLibraryAlbumsContinuation(LibraryContinuationRequest) returns (LibraryAlbumsResponse);
  rpc GetLibrarySubscriptions(Empty) returns (LibrarySubscriptionsResponse);
  rpc GetLibrarySubscriptionsContinuation(LibraryContinuationRequest) returns (LibrarySubscriptionsResponse);
  rpc GetLibraryChannels(Empty) returns (LibraryChannelsResponse);
  rpc GetLibraryChannelsContinuation(LibraryContinuationRequest) returns (LibraryChannelsResponse);
  rpc GetLibraryPodcasts(Empty) returns (LibraryPodcastsResponse);
  rpc GetLibraryPodcastsContinuation(LibraryContinuationRequest) returns (LibraryPodcastsResponse);
  rpc GetLibrarySongs(Empty) returns (LibrarySongsResponse);
  rpc GetLibrarySongsContinuation(LibraryContinuationRequest) returns (LibrarySongsResponse);
  rpc GetLikedSongs(Empty) returns (LikedSongsResponse);
  rpc GetLikedSongsContinuation(LibraryContinuationRequest) returns (LikedSongsResponse);
  rpc GetSavedEpisodes(Empty) returns (SavedEpisodesResponse);
  rpc GetSavedEpisodesContinuation(LibraryContinuationRequest) returns (SavedEpisodesResponse);
  rpc Decipher(DecipherRequest) returns (DecipherResponse);
}

message Empty {}
message SearchRequest { string query = 1; string filter = 2; bool ignore_spelling = 3; }
message SearchContinuationRequest { string token = 1; }
message GetWatchPlaylistRequest { string video_id = 1; string playlist_id = 2; bool radio = 3; bool shuffle = 4; }
message GetWatchPlaylistContinuationRequest { string token = 1; }
message GetSongRequest { string video_id = 1; }
message LibraryContinuationRequest { string token = 1; }
message DecipherRequest { string signature_cipher = 1; }
message DecipherResponse { string playable_url = 1; }
message SearchResultItem { string kind = 1; string title = 2; string browse_id = 3; string video_id = 4; }
message ContinuationToken { string value = 1; }
message SearchResponse { repeated SearchResultItem items = 1; ContinuationToken continuation = 2; }
message WatchTrackItem { string video_id = 1; string title = 2; string playlist_id = 3; }
message WatchPlaylistResponse { repeated WatchTrackItem items = 1; ContinuationToken continuation = 2; }
message GetSongResponse { string video_id = 1; repeated SongStreamFormat formats = 2; repeated SongStreamFormat adaptive_formats = 3; }
message SongStreamFormat { uint32 itag = 1; string mime_type = 2; string signature_cipher = 3; }
message LibraryPlaylistItem { string playlist_id = 1; string title = 2; }
message LibraryPlaylistsResponse { repeated LibraryPlaylistItem items = 1; ContinuationToken continuation = 2; }
message AccountInfoResponse { string account_name = 1; }
message LibraryArtistItem { string artist_id = 1; string name = 2; }
message LibraryArtistsResponse { repeated LibraryArtistItem items = 1; ContinuationToken continuation = 2; }
message LibraryAlbumItem { string album_id = 1; string title = 2; }
message LibraryAlbumsResponse { repeated LibraryAlbumItem items = 1; ContinuationToken continuation = 2; }
message LibrarySubscriptionItem { string channel_id = 1; string title = 2; }
message LibrarySubscriptionsResponse { repeated LibrarySubscriptionItem items = 1; ContinuationToken continuation = 2; }
message LibraryChannelItem { string channel_id = 1; string title = 2; }
message LibraryChannelsResponse { repeated LibraryChannelItem items = 1; ContinuationToken continuation = 2; }
message LibraryPodcastItem { string podcast_id = 1; string title = 2; }
message LibraryPodcastsResponse { repeated LibraryPodcastItem items = 1; ContinuationToken continuation = 2; }
message LibrarySongItem { string video_id = 1; string title = 2; }
message LibrarySongsResponse { repeated LibrarySongItem items = 1; ContinuationToken continuation = 2; }
message LikedSongItem { string video_id = 1; string title = 2; }
message LikedSongsResponse { repeated LikedSongItem items = 1; ContinuationToken continuation = 2; }
message SavedEpisodeItem { string video_id = 1; string title = 2; }
message SavedEpisodesResponse { repeated SavedEpisodeItem items = 1; ContinuationToken continuation = 2; }
```

```proto
syntax = "proto3";

package ytmusic.v1.admin;

service YtMusicAdmin {
  rpc ReloadBrowserAuth(ReloadBrowserAuthRequest) returns (ReloadBrowserAuthResponse);
}

message ReloadBrowserAuthRequest {}
message ReloadBrowserAuthResponse {
  string active_version = 1;
}
```

- [ ] **Step 2: Run compile check to verify code generation fails before build wiring exists**

Run: `cargo check`
Expected: FAIL because generated modules are not compiled into the crate yet.

- [ ] **Step 3: Wire `build.rs` and generated module includes**

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = std::path::PathBuf::from(std::env::var("OUT_DIR")?)
        .join("ytmusic_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &["proto/ytmusic/v1/public.proto", "proto/ytmusic/v1/admin.proto"],
            &["proto"],
        )?;

    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
```

```rust
// src/proto.rs
pub mod ytmusic {
    pub mod v1 {
        tonic::include_proto!("ytmusic.v1");
        pub const FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("ytmusic_descriptor");
    }

    pub mod admin {
        tonic::include_proto!("ytmusic.v1.admin");
    }
}
```

- [ ] **Step 4: Expand the minimal response shapes to match upstream models**

```proto
message Thumbnail {
  string url = 1;
  uint32 width = 2;
  uint32 height = 3;
}

message ContinuationToken {
  string value = 1;
}

message SearchResultItem {
  string kind = 1;
  string title = 2;
  string browse_id = 3;
  string video_id = 4;
}

message SearchResponse {
  repeated SearchResultItem items = 1;
  ContinuationToken continuation = 2;
}
```

Add the same explicit pattern for watch, song, library, liked songs, and saved episodes response messages in `public.proto`. Expand the minimal field sets above until every upstream response field you intend to expose has a named protobuf field.

- [ ] **Step 5: Run compile check to verify generated types build**

Run: `cargo check`
Expected: FAIL only because the service implementation modules still do not exist.

- [ ] **Step 6: Commit**

```bash
git add build.rs proto/ytmusic/v1/public.proto proto/ytmusic/v1/admin.proto src/proto.rs
git commit -m "feat: define gRPC protobuf contracts"
```

### Task 3: Add Configuration, Service Errors, and Startup Validation Tests

**Files:**
- Create: `src/config.rs`
- Create: `src/error.rs`
- Create: `tests/startup.rs`
- Modify: `src/lib.rs`
- Test: `cargo test startup -- --nocapture`

- [ ] **Step 1: Write the failing startup tests**

```rust
// tests/startup.rs
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
```

- [ ] **Step 2: Run the startup test to verify it fails**

Run: `cargo test startup_fails_when_browser_json_is_missing -- --nocapture`
Expected: FAIL because `ServiceConfig` does not exist yet.

- [ ] **Step 3: Implement configuration parsing and local error types**

```rust
// src/config.rs
use std::{net::SocketAddr, path::PathBuf};

use crate::error::ServiceError;

#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub public_addr: SocketAddr,
    pub admin_addr: SocketAddr,
    pub browser_auth_path: PathBuf,
}

impl ServiceConfig {
    pub fn from_parts(
        public_addr: &str,
        admin_addr: &str,
        browser_auth_path: PathBuf,
    ) -> Result<Self, ServiceError> {
        if !browser_auth_path.exists() {
            return Err(ServiceError::BrowserAuthPathMissing(browser_auth_path));
        }

        Ok(Self {
            public_addr: public_addr.parse()?,
            admin_addr: admin_addr.parse()?,
            browser_auth_path,
        })
    }
}
```

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("browser auth file does not exist: {0}")]
    BrowserAuthPathMissing(std::path::PathBuf),
    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] std::net::AddrParseError),
}
```

Update `Cargo.toml` to add:

```toml
thiserror = "2"
```

- [ ] **Step 4: Export the new modules and run the test**

```rust
// src/lib.rs
pub mod config;
pub mod error;
```

Run: `cargo test startup_fails_when_browser_json_is_missing -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Add the valid-file startup test**

```rust
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
```

Run: `cargo test startup -- --nocapture`
Expected: PASS for both startup path validation tests.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/config.rs src/error.rs src/lib.rs tests/startup.rs
git commit -m "feat: add startup config validation"
```

### Task 4: Implement AuthContext Construction and `ArcSwap` Runtime State

**Files:**
- Create: `src/auth_context.rs`
- Create: `src/state.rs`
- Modify: `src/error.rs`
- Modify: `tests/startup.rs`
- Test: `cargo test startup_requires_valid_browser_auth_json -- --nocapture`

- [ ] **Step 1: Write the failing auth validation test**

```rust
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

    let result = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config)
        .await;

    assert!(result.is_err());
}
```

- [ ] **Step 2: Run the new test to verify it fails**

Run: `cargo test startup_requires_valid_browser_auth_json -- --nocapture`
Expected: FAIL because `AuthContext` does not exist yet.

- [ ] **Step 3: Implement `AuthContext` and shared state**

```rust
// src/auth_context.rs
use std::{path::Path, sync::Arc, time::SystemTime};

use crate::{config::ServiceConfig, error::ServiceError};

#[derive(Clone)]
pub struct AuthContext {
    pub client: ytmusicapi::YtMusic,
    pub version: Arc<str>,
    pub loaded_at: SystemTime,
}

impl AuthContext {
    pub async fn from_browser_auth_file(config: &ServiceConfig) -> Result<Self, ServiceError> {
        let client = ytmusicapi::YtMusic::from_browser_auth_file(&config.browser_auth_path)
            .map_err(ServiceError::BrowserAuthLoad)?;

        Ok(Self {
            client,
            version: Arc::<str>::from(version_from_path(&config.browser_auth_path)),
            loaded_at: SystemTime::now(),
        })
    }
}

fn version_from_path(path: &Path) -> String {
    format!("{}:{}", path.display(), chrono::Utc::now().timestamp())
}
```

```rust
// src/state.rs
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::auth_context::AuthContext;

pub struct AppState {
    pub auth: ArcSwap<AuthContext>,
    pub cipher: Arc<yt_cipher::YtCipher>,
}

impl AppState {
    pub fn new(auth: AuthContext, cipher: yt_cipher::YtCipher) -> Self {
        Self {
            auth: ArcSwap::from_pointee(auth),
            cipher: Arc::new(cipher),
        }
    }
}
```

Add to `Cargo.toml`:

```toml
chrono = { version = "0.4", default-features = false, features = ["clock"] }
```

- [ ] **Step 4: Extend `ServiceError` for browser auth loading**

```rust
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("browser auth file does not exist: {0}")]
    BrowserAuthPathMissing(std::path::PathBuf),
    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] std::net::AddrParseError),
    #[error("failed to load browser auth: {0}")]
    BrowserAuthLoad(#[source] ytmusicapi::Error),
}
```

- [ ] **Step 5: Run the auth validation tests**

Run: `cargo test startup_requires_valid_browser_auth_json -- --nocapture`
Expected: PASS with an auth validation error coming from `ytmusicapi`.

Run: `cargo test startup -- --nocapture`
Expected: PASS for all startup tests.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/auth_context.rs src/state.rs src/error.rs tests/startup.rs
git commit -m "feat: add auth context and shared runtime state"
```

### Task 5: Build the `ytmusicapi` and `yt-cipher` Adapter Layer

**Files:**
- Create: `src/adapters/mod.rs`
- Create: `src/adapters/ytmusic.rs`
- Create: `src/adapters/cipher.rs`
- Modify: `src/error.rs`
- Test: `cargo check`

- [ ] **Step 1: Write the failing adapter trait shells**

```rust
// src/adapters/mod.rs
pub mod cipher;
pub mod ytmusic;
```

```rust
// src/adapters/ytmusic.rs
use crate::{auth_context::AuthContext, error::ServiceError};

pub struct YtMusicAdapter;

impl YtMusicAdapter {
    pub async fn get_account_info(
        auth: &AuthContext,
    ) -> Result<ytmusicapi::AccountInfo, ServiceError> {
        auth.client.get_account_info().await.map_err(ServiceError::YtMusic)
    }
}
```

```rust
// src/adapters/cipher.rs
use crate::error::ServiceError;

pub struct CipherAdapter;

impl CipherAdapter {
    pub async fn decipher(
        cipher: &yt_cipher::YtCipher,
        raw: &str,
    ) -> Result<String, ServiceError> {
        cipher.decipher(raw).await.map_err(ServiceError::Cipher)
    }
}
```

- [ ] **Step 2: Run compile check to verify error variants are missing**

Run: `cargo check`
Expected: FAIL because `ServiceError::Cipher` and adapter return types are incomplete.

- [ ] **Step 3: Add adapter return types and error conversions**

```rust
// src/adapters/ytmusic.rs
use crate::{auth_context::AuthContext, error::ServiceError};

pub struct YtMusicAdapter;

impl YtMusicAdapter {
    pub async fn search(
        auth: &AuthContext,
        query: ytmusicapi::SearchQuery,
    ) -> Result<ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>, ServiceError> {
        auth.client.search(query).await.map_err(ServiceError::YtMusic)
    }

    pub async fn get_song(
        auth: &AuthContext,
        video_id: String,
        signature_timestamp: u32,
    ) -> Result<ytmusicapi::GetSongResponse, ServiceError> {
        auth.client
            .get_song(video_id, signature_timestamp)
            .await
            .map_err(ServiceError::YtMusic)
    }

    pub async fn get_account_info(
        auth: &AuthContext,
    ) -> Result<ytmusicapi::AccountInfo, ServiceError> {
        auth.client.get_account_info().await.map_err(ServiceError::YtMusic)
    }
}
```

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("browser auth file does not exist: {0}")]
    BrowserAuthPathMissing(std::path::PathBuf),
    #[error("invalid socket address: {0}")]
    InvalidSocketAddress(#[from] std::net::AddrParseError),
    #[error("failed to load browser auth: {0}")]
    BrowserAuthLoad(#[source] ytmusicapi::Error),
    #[error("ytmusicapi request failed: {0}")]
    YtMusic(#[source] ytmusicapi::Error),
    #[error("yt-cipher request failed: {0}")]
    Cipher(#[source] yt_cipher::Error),
}
```

- [ ] **Step 4: Flesh out one-to-one adapter methods for all upstream endpoints**

Add the same explicit wrapper pattern to `src/adapters/ytmusic.rs` for these concrete methods:

```rust
pub async fn search_continuation(auth: &AuthContext, token: ytmusicapi::SearchContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>, ServiceError>
pub async fn get_watch_playlist(auth: &AuthContext, query: ytmusicapi::WatchPlaylistQuery) -> Result<ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>, ServiceError>
pub async fn get_watch_playlist_continuation(auth: &AuthContext, token: ytmusicapi::WatchPlaylistContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>, ServiceError>
pub async fn get_library_playlists(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibraryPlaylist, ytmusicapi::LibraryPlaylistsContinuationToken>, ServiceError>
pub async fn get_library_playlists_continuation(auth: &AuthContext, token: ytmusicapi::LibraryPlaylistsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibraryPlaylist, ytmusicapi::LibraryPlaylistsContinuationToken>, ServiceError>
pub async fn get_library_artists(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibraryArtist, ytmusicapi::LibraryArtistsContinuationToken>, ServiceError>
pub async fn get_library_artists_continuation(auth: &AuthContext, token: ytmusicapi::LibraryArtistsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibraryArtist, ytmusicapi::LibraryArtistsContinuationToken>, ServiceError>
pub async fn get_library_albums(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibraryAlbum, ytmusicapi::LibraryAlbumsContinuationToken>, ServiceError>
pub async fn get_library_albums_continuation(auth: &AuthContext, token: ytmusicapi::LibraryAlbumsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibraryAlbum, ytmusicapi::LibraryAlbumsContinuationToken>, ServiceError>
pub async fn get_library_subscriptions(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibrarySubscription, ytmusicapi::LibrarySubscriptionsContinuationToken>, ServiceError>
pub async fn get_library_subscriptions_continuation(auth: &AuthContext, token: ytmusicapi::LibrarySubscriptionsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibrarySubscription, ytmusicapi::LibrarySubscriptionsContinuationToken>, ServiceError>
pub async fn get_library_channels(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibraryChannel, ytmusicapi::LibraryChannelsContinuationToken>, ServiceError>
pub async fn get_library_channels_continuation(auth: &AuthContext, token: ytmusicapi::LibraryChannelsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibraryChannel, ytmusicapi::LibraryChannelsContinuationToken>, ServiceError>
pub async fn get_library_podcasts(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibraryPodcast, ytmusicapi::LibraryPodcastsContinuationToken>, ServiceError>
pub async fn get_library_podcasts_continuation(auth: &AuthContext, token: ytmusicapi::LibraryPodcastsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibraryPodcast, ytmusicapi::LibraryPodcastsContinuationToken>, ServiceError>
pub async fn get_library_songs(auth: &AuthContext) -> Result<ytmusicapi::Page<ytmusicapi::LibrarySong, ytmusicapi::LibrarySongsContinuationToken>, ServiceError>
pub async fn get_library_songs_continuation(auth: &AuthContext, token: ytmusicapi::LibrarySongsContinuationToken) -> Result<ytmusicapi::Page<ytmusicapi::LibrarySong, ytmusicapi::LibrarySongsContinuationToken>, ServiceError>
pub async fn get_liked_songs(auth: &AuthContext) -> Result<ytmusicapi::LikedSongsPage, ServiceError>
pub async fn get_liked_songs_continuation(auth: &AuthContext, token: ytmusicapi::LikedSongsContinuationToken) -> Result<ytmusicapi::LikedSongsPage, ServiceError>
pub async fn get_saved_episodes(auth: &AuthContext) -> Result<ytmusicapi::SavedEpisodesPage, ServiceError>
pub async fn get_saved_episodes_continuation(auth: &AuthContext, token: ytmusicapi::SavedEpisodesContinuationToken) -> Result<ytmusicapi::SavedEpisodesPage, ServiceError>
```

Each wrapper should directly call the corresponding upstream method and return `Result<UpstreamType, ServiceError>`.

- [ ] **Step 5: Run compile check**

Run: `cargo check`
Expected: PASS for adapter and error wiring, even though gRPC servers are not implemented yet.

- [ ] **Step 6: Commit**

```bash
git add src/adapters/mod.rs src/adapters/ytmusic.rs src/adapters/cipher.rs src/error.rs
git commit -m "feat: add upstream adapter layer"
```

### Task 6: Implement the Public gRPC Service for Search, Watch, Song, and Decipher

**Files:**
- Create: `src/servers/mod.rs`
- Create: `src/servers/public.rs`
- Modify: `src/error.rs`
- Create: `tests/public_api.rs`
- Test: `cargo test public_search_rejects_empty_query -- --nocapture`

- [ ] **Step 1: Write the failing public API test**

```rust
// tests/public_api.rs
#[tokio::test]
async fn public_search_rejects_empty_query() {
    let status = ytmusic_service::error::map_invalid_argument("query must not be empty");
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
```

- [ ] **Step 2: Run the public API test to verify it fails**

Run: `cargo test public_search_rejects_empty_query -- --nocapture`
Expected: FAIL because the mapper helper does not exist yet.

- [ ] **Step 3: Add gRPC status mapping helpers and public service skeleton**

```rust
// src/error.rs
pub fn map_invalid_argument(message: impl Into<String>) -> tonic::Status {
    tonic::Status::invalid_argument(message.into())
}

pub fn map_service_error(error: &ServiceError) -> tonic::Status {
    match error {
        ServiceError::YtMusic(source) => tonic::Status::unavailable(source.to_string()),
        ServiceError::Cipher(source) => tonic::Status::internal(source.to_string()),
        ServiceError::BrowserAuthPathMissing(path) => {
            tonic::Status::failed_precondition(format!("browser auth file missing: {}", path.display()))
        }
        _ => tonic::Status::internal(error.to_string()),
    }
}
```

```rust
// src/servers/mod.rs
pub mod admin;
pub mod public;
```

```rust
// src/servers/public.rs
use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    adapters::{cipher::CipherAdapter, ytmusic::YtMusicAdapter},
    proto::ytmusic::v1::{
        y_t_music_public_server::YtMusicPublic, DecipherRequest, DecipherResponse,
        GetSongRequest, GetSongResponse, SearchRequest, SearchResponse,
    },
    state::AppState,
};

pub struct PublicService {
    pub state: Arc<AppState>,
}

#[tonic::async_trait]
impl YtMusicPublic for PublicService {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let request = request.into_inner();
        if request.query.trim().is_empty() {
            return Err(crate::error::map_invalid_argument("query must not be empty"));
        }
        Err(Status::failed_precondition("search adapter wiring has not been added yet"))
    }

    async fn get_song(
        &self,
        _request: Request<GetSongRequest>,
    ) -> Result<Response<GetSongResponse>, Status> {
        Err(Status::failed_precondition("song adapter wiring has not been added yet"))
    }

    async fn decipher(
        &self,
        request: Request<DecipherRequest>,
    ) -> Result<Response<DecipherResponse>, Status> {
        let auth = self.state.auth.load();
        let _ = auth.version.clone();
        let url = CipherAdapter::decipher(&self.state.cipher, &request.into_inner().signature_cipher)
            .await
            .map_err(|e| crate::error::map_service_error(&e))?;
        Ok(Response::new(DecipherResponse { playable_url: url }))
    }
}
```

- [ ] **Step 4: Run the focused test**

Run: `cargo test public_search_rejects_empty_query -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Fill in explicit one-to-one implementations for the public endpoints**

In `src/servers/public.rs`, replace the temporary failed-precondition branches with concrete handlers for:

```rust
search
search_continuation
get_watch_playlist
get_watch_playlist_continuation
get_song
decipher
```

Handler pattern:

```rust
let auth = self.state.auth.load();
let query = ytmusicapi::SearchQuery::new(request.query);
let page = YtMusicAdapter::search(&auth, query).await.map_err(|e| crate::error::map_service_error(&e))?;
let response = crate::servers::public::mapping::search_page_to_proto(page);
Ok(Response::new(response))
```

Also add a nested `mapping` module inside `src/servers/public.rs` or split to `src/servers/public_mapping.rs` if the file exceeds comfortable size. Keep all mapper function names explicit:

```rust
fn search_page_to_proto(page: ytmusicapi::Page<ytmusicapi::SearchResult, ytmusicapi::SearchContinuationToken>) -> SearchResponse
fn watch_page_to_proto(page: ytmusicapi::Page<ytmusicapi::WatchTrack, ytmusicapi::WatchPlaylistContinuationToken>) -> WatchPlaylistResponse
fn song_response_to_proto(song: ytmusicapi::GetSongResponse) -> GetSongResponse
```

- [ ] **Step 6: Run service tests**

Run: `cargo test public_search_rejects_empty_query -- --nocapture`
Expected: PASS.

Run: `cargo check`
Expected: PASS for the public service implementation.

- [ ] **Step 7: Commit**

```bash
git add src/error.rs src/servers/mod.rs src/servers/public.rs tests/public_api.rs
git commit -m "feat: add public search watch song and decipher RPCs"
```

### Task 7: Implement the Remaining Library and Account RPCs

**Files:**
- Modify: `src/servers/public.rs`
- Modify: `proto/ytmusic/v1/public.proto`
- Modify: `tests/public_api.rs`
- Test: `cargo check`

- [ ] **Step 1: Write the failing account-info mapping test**

```rust
#[tokio::test]
async fn account_info_response_keeps_name_field() {
    let response = ytmusic_service::proto::ytmusic::v1::AccountInfoResponse {
        account_name: "listener@example.com".to_owned(),
    };

    assert_eq!(response.account_name, "listener@example.com");
}
```

- [ ] **Step 2: Run the test to verify the typed field exists**

Run: `cargo test account_info_response_keeps_name_field -- --nocapture`
Expected: FAIL if `AccountInfoResponse` is still underspecified.

- [ ] **Step 3: Complete the protobuf messages for library and account surfaces**

In `proto/ytmusic/v1/public.proto`, replace any remaining coarse fields with dedicated typed messages for:

```proto
AccountInfoResponse
LibraryPlaylistItem
LibraryArtistItem
LibraryAlbumItem
LibrarySubscriptionItem
LibraryChannelItem
LibraryPodcastItem
LibrarySongItem
LikedSongItem
SavedEpisodeItem
```

Use the upstream Rust model fields as the source of truth for names and cardinality.

- [ ] **Step 4: Implement the remaining public handlers**

In `src/servers/public.rs`, add one concrete handler per upstream method:

```rust
get_library_playlists
get_library_playlists_continuation
get_account_info
get_library_artists
get_library_artists_continuation
get_library_albums
get_library_albums_continuation
get_library_subscriptions
get_library_subscriptions_continuation
get_library_channels
get_library_channels_continuation
get_library_podcasts
get_library_podcasts_continuation
get_library_songs
get_library_songs_continuation
get_liked_songs
get_liked_songs_continuation
get_saved_episodes
get_saved_episodes_continuation
```

Use the same load-auth, call-adapter, map-response pattern as Task 6. Do not invent generic dispatch helpers.

- [ ] **Step 5: Run build and tests**

Run: `cargo test account_info_response_keeps_name_field -- --nocapture`
Expected: PASS.

Run: `cargo check`
Expected: PASS with the full one-to-one public service surface implemented.

- [ ] **Step 6: Commit**

```bash
git add proto/ytmusic/v1/public.proto src/servers/public.rs tests/public_api.rs
git commit -m "feat: add remaining library and account RPCs"
```

### Task 8: Implement the Admin Service and Atomic Reload Flow

**Files:**
- Create: `src/servers/admin.rs`
- Modify: `src/auth_context.rs`
- Modify: `src/state.rs`
- Create: `tests/reload.rs`
- Test: `cargo test reload_keeps_previous_context_on_validation_failure -- --nocapture`

- [ ] **Step 1: Write the failing reload test**

```rust
// tests/reload.rs
use std::sync::Arc;

use arc_swap::ArcSwap;

#[test]
fn reload_keeps_previous_context_on_validation_failure() {
    let initial = Arc::new("v1".to_owned());
    let swap = ArcSwap::from(initial.clone());

    let loaded = swap.load();
    assert_eq!(loaded.as_str(), "v1");
}
```

- [ ] **Step 2: Run the reload test**

Run: `cargo test reload_keeps_previous_context_on_validation_failure -- --nocapture`
Expected: PASS for `ArcSwap` semantics but no service reload behavior exists yet.

- [ ] **Step 3: Add `AuthContext` probe validation and swap helpers**

```rust
// src/auth_context.rs
impl AuthContext {
    pub async fn probe(&self) -> Result<(), ServiceError> {
        self.client
            .get_account_info()
            .await
            .map(|_| ())
            .map_err(ServiceError::YtMusic)
    }
}
```

```rust
// src/state.rs
impl AppState {
    pub async fn reload_browser_auth(
        &self,
        config: &crate::config::ServiceConfig,
    ) -> Result<String, crate::error::ServiceError> {
        let next = crate::auth_context::AuthContext::from_browser_auth_file(config).await?;
        next.probe().await?;
        let version = next.version.to_string();
        self.auth.store(Arc::new(next));
        Ok(version)
    }
}
```

- [ ] **Step 4: Implement the admin tonic service**

```rust
// src/servers/admin.rs
use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::{
    config::ServiceConfig,
    proto::ytmusic::admin::{
        y_t_music_admin_server::YtMusicAdmin, ReloadBrowserAuthRequest,
        ReloadBrowserAuthResponse,
    },
    state::AppState,
};

pub struct AdminService {
    pub state: Arc<AppState>,
    pub config: ServiceConfig,
}

#[tonic::async_trait]
impl YtMusicAdmin for AdminService {
    async fn reload_browser_auth(
        &self,
        _request: Request<ReloadBrowserAuthRequest>,
    ) -> Result<Response<ReloadBrowserAuthResponse>, Status> {
        let version = self
            .state
            .reload_browser_auth(&self.config)
            .await
            .map_err(|e| crate::error::map_service_error(&e))?;

        Ok(Response::new(ReloadBrowserAuthResponse {
            active_version: version,
        }))
    }
}
```

- [ ] **Step 5: Extend reload tests to cover success and failure paths**

Add test cases in `tests/reload.rs` for:

```rust
#[tokio::test]
async fn reload_swaps_context_after_successful_probe() {
    let old_version = state.auth.load().version.to_string();
    std::fs::write(&config.browser_auth_path, valid_browser_auth_json()).unwrap();
    let new_version = state.reload_browser_auth(&config).await.unwrap();
    assert_ne!(old_version, new_version);
}

#[tokio::test]
async fn reload_failure_preserves_prior_version() {
    let old_version = state.auth.load().version.to_string();
    std::fs::write(&config.browser_auth_path, "{\"cookie\":\"broken\"}").unwrap();
    let result = state.reload_browser_auth(&config).await;
    assert!(result.is_err());
    assert_eq!(state.auth.load().version.as_ref(), old_version);
}
```

Use `AppState::reload_browser_auth` directly in tests instead of spinning up gRPC transport first.

- [ ] **Step 6: Run reload tests**

Run: `cargo test reload -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/auth_context.rs src/state.rs src/servers/admin.rs tests/reload.rs
git commit -m "feat: add admin auth reload service"
```

### Task 9: Wire Process Startup, Dual Listeners, Health, and Reflection

**Files:**
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Modify: `src/config.rs`
- Test: `cargo check`

- [ ] **Step 1: Write the failing bootstrap function signature**

```rust
// src/lib.rs
pub async fn run(_config: config::ServiceConfig) -> Result<(), error::ServiceError> {
    Ok(())
}
```

- [ ] **Step 2: Run compile check**

Run: `cargo check`
Expected: PASS with the temporary no-op `run` function present, but the binary still does not run the server.

- [ ] **Step 3: Implement startup wiring in the library**

```rust
// src/lib.rs
pub async fn run(config: config::ServiceConfig) -> Result<(), error::ServiceError> {
    let auth = auth_context::AuthContext::from_browser_auth_file(&config).await?;
    auth.probe().await?;

    let cipher = yt_cipher::YtCipher::create()
        .await
        .map_err(error::ServiceError::Cipher)?;

    let state = std::sync::Arc::new(state::AppState::new(auth, cipher));

    let public_service = servers::public::PublicService {
        state: state.clone(),
    };
    let admin_service = servers::admin::AdminService {
        state: state.clone(),
        config: config.clone(),
    };

    let (mut reporter, health) = tonic_health::server::health_reporter();
    reporter
        .set_serving::<proto::ytmusic::v1::y_t_music_public_server::YtMusicPublicServer<servers::public::PublicService>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::ytmusic::v1::FILE_DESCRIPTOR_SET)
        .build_v1()
        .map_err(error::ServiceError::Reflection)?;

    let public = tonic::transport::Server::builder()
        .add_service(health.clone())
        .add_service(proto::ytmusic::v1::y_t_music_public_server::YtMusicPublicServer::new(public_service))
        .serve(config.public_addr);

    let admin = tonic::transport::Server::builder()
        .add_service(health)
        .add_service(reflection)
        .add_service(proto::ytmusic::admin::y_t_music_admin_server::YtMusicAdminServer::new(admin_service))
        .serve(config.admin_addr);

    tokio::try_join!(public, admin).map_err(error::ServiceError::Transport)?;
    Ok(())
}
```

- [ ] **Step 4: Update `main.rs` to read config and start the runtime**

```rust
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ytmusic_service::config::ServiceConfig::from_env()?;
    ytmusic_service::run(config).await?;
    Ok(())
}
```

Add `from_env()` in `src/config.rs` using:

```rust
std::env::var("YTMUSIC_SERVICE_PUBLIC_ADDR")
std::env::var("YTMUSIC_SERVICE_ADMIN_ADDR")
std::env::var("YTMUSIC_SERVICE_BROWSER_JSON")
```

- [ ] **Step 5: Run build verification**

Run: `cargo check`
Expected: PASS with both server listeners wired.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/main.rs src/config.rs
git commit -m "feat: wire dual-listener service startup"
```

### Task 10: Add End-to-End Service Tests, Container Packaging, and Operator Docs

**Files:**
- Modify: `tests/public_api.rs`
- Modify: `tests/startup.rs`
- Modify: `tests/reload.rs`
- Create: `Dockerfile`
- Create: `.dockerignore`
- Create: `README.md`
- Test: `cargo test`

- [ ] **Step 1: Add startup and reload test coverage gaps**

Add these cases:

```rust
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
    assert!(result.is_err());
}

#[tokio::test]
async fn startup_fails_when_browser_json_probe_fails() {
    let config = test_config_with_unusable_but_well_formed_browser_auth();
    let context = ytmusic_service::auth_context::AuthContext::from_browser_auth_file(&config).await.unwrap();
    let result = context.probe().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn in_flight_request_keeps_old_context_during_reload() {
    let before = state.auth.load();
    let before_version = before.version.to_string();
    state.auth.store(std::sync::Arc::new(next_context));
    assert_eq!(before.version.as_ref(), before_version);
    assert_ne!(state.auth.load().version.as_ref(), before_version);
}
```

- [ ] **Step 2: Run the full Rust test suite**

Run: `cargo test`
Expected: PASS for startup, public API, and reload coverage.

- [ ] **Step 3: Add the container files**

```dockerfile
# Dockerfile
FROM rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY proto ./proto
COPY src ./src
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/ytmusic-service /app/ytmusic-service
USER nonroot:nonroot
EXPOSE 50051 50052
ENTRYPOINT ["/app/ytmusic-service"]
```

```dockerignore
/target
/.git
/.superpowers
/.serena
```

- [ ] **Step 4: Write the operator README**

```md
# ytmusic-service

## Required environment

- `YTMUSIC_SERVICE_PUBLIC_ADDR`
- `YTMUSIC_SERVICE_ADMIN_ADDR`
- `YTMUSIC_SERVICE_BROWSER_JSON`

## Local run

```bash
cargo run
```

## Container run

```bash
podman build -t ytmusic-service .
podman run --rm \
  -p 50051:50051 \
  -p 50052:50052 \
  -e YTMUSIC_SERVICE_PUBLIC_ADDR=0.0.0.0:50051 \
  -e YTMUSIC_SERVICE_ADMIN_ADDR=0.0.0.0:50052 \
  -e YTMUSIC_SERVICE_BROWSER_JSON=/run/secrets/browser.json \
  -v ./browser.json:/run/secrets/browser.json:ro \
  ytmusic-service
```

## Admin reload

Replace the mounted `browser.json`, then call `ReloadBrowserAuth` against the admin listener.
```
```

- [ ] **Step 5: Run verification commands**

Run: `cargo test`
Expected: PASS.

Run: `docker build -t ytmusic-service .`
Expected: PASS with a distroless final stage.

- [ ] **Step 6: Commit**

```bash
git add tests/startup.rs tests/public_api.rs tests/reload.rs Dockerfile .dockerignore README.md
git commit -m "feat: add tests container packaging and operator docs"
```

## Self-Review

- Spec coverage:
  - dual-listener architecture: Tasks 8 and 9
  - strict `browser.json` startup requirement: Tasks 3, 4, and 10
  - admin-only explicit reload: Task 8
  - `ArcSwap` atomic state cutover: Tasks 4 and 8
  - one-to-one typed public RPC surface: Tasks 2, 5, 6, and 7
  - separate `Decipher` RPC: Tasks 2, 5, and 6
  - distroless container packaging: Task 10
- Placeholder scan:
  - remove every temporary failed-precondition branch and every minimal proto field set before closing Tasks 6 and 7
- Type consistency:
  - keep protobuf service names as `YtMusicPublic` and `YtMusicAdmin`
  - keep shared reload entrypoint name as `AppState::reload_browser_auth`
  - keep auth validation probe method name as `AuthContext::probe`
