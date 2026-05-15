# ytmusic-service Design

Date: 2026-05-16

## Goal

Build `ytmusic-service` as a Rust microservice using gRPC and `tonic`.

The service must:
- wrap all currently exposed `ytmusicapi` capabilities from `https://github.com/ghfhffh12345/ytmusicapi.git` tag `v0.1.0`
- use `https://github.com/ghfhffh12345/yt-cipher.git` tag `v0.1.0`
- expose a fully typed protobuf API
- map upstream `ytmusicapi` methods one-to-one into gRPC RPCs
- require a valid `browser.json` at startup
- support explicit admin-triggered auth reload via gRPC
- expose deciphering as a separate RPC that accepts a caller-selected `signatureCipher`
- package cleanly for Podman and Docker using a distroless final runtime image

## Non-Goals

- The service does not choose a preferred stream variant for callers.
- The service does not collapse multiple upstream methods into generic RPCs.
- The service does not auto-watch `browser.json`.
- The service does not accept startup without browser authentication.

## External Dependencies

### Upstream libraries

- `ytmusicapi` tag `v0.1.0`
  - primary features observed from the crate surface:
    - `search`
    - `search_continuation`
    - `get_watch_playlist`
    - `get_watch_playlist_continuation`
    - `get_song`
    - library/account endpoints and their continuation methods
    - browser auth setup helper
- `yt-cipher` tag `v0.1.0`
  - primary features observed from the crate surface:
    - eager player bootstrap
    - `signature_timestamp`
    - `decipher`
    - explicit player refresh support

### Runtime and platform

- Rust
- Tokio
- `tonic` for gRPC
- `prost` / `tonic-build` for protobuf generation
- `ArcSwap` for atomic auth-context replacement
- Podman/Docker-compatible container packaging

## Architecture

`ytmusic-service` will run as a single Rust binary with two gRPC listeners in the same Tokio runtime:

- public listener
  - exposes caller-facing RPCs corresponding to `ytmusicapi` plus `Decipher`
- admin listener
  - binds to a separate internal address/network
  - exposes only control-plane methods such as `ReloadBrowserAuth`, health, and reflection

This is the selected "single binary, dual listener" architecture.

### Internal modules

- `proto`
  - protobuf definitions
  - generated code
  - descriptor set for reflection
- `config`
  - CLI/env configuration parsing
  - public/admin listen addresses
  - `browser.json` path
  - timeouts and logging settings
- `app/state`
  - shared runtime state
  - `ArcSwap<Arc<AuthContext>>`
  - long-lived `YtCipher`
  - health and shutdown coordination
- `adapters/ytmusic`
  - thin wrapper around the `ytmusicapi` crate
  - request/response mapping between domain types and protobuf types
- `adapters/cipher`
  - thin wrapper around the `yt-cipher` crate
  - decipher-specific validation and mapping
- `servers/public`
  - public tonic services
  - request interceptors
  - error translation
- `servers/admin`
  - admin tonic services
  - reload endpoint
  - reflection and health wiring

The gRPC layer should not directly embed upstream client logic; it should call the adapter layer so request handling, mapping, and runtime state remain testable and modular.

## Authentication Model

### Startup requirement

`browser.json` is mandatory.

Before either gRPC listener binds, the service must:

1. read the configured `browser.json`
2. parse and normalize it
3. validate it against the same browser-auth rules expected by `ytmusicapi`
4. construct an initial authenticated `YtMusic` client context
5. perform a lightweight authenticated validation probe before serving traffic

If any step fails because the file is missing, malformed, invalid, or unusable for authenticated requests, the process must fail startup.

### Runtime reload

`browser.json` rotation is explicit and admin-driven.

Operational flow:

1. administrator replaces the mounted `browser.json`
2. administrator calls `ReloadBrowserAuth` on the admin listener
3. the service reads and validates the replacement file
4. the service builds a fresh `AuthContext` off to the side
5. the service performs a lightweight authenticated probe with the new context
6. if validation succeeds, the service atomically swaps the active context
7. if validation fails, the previous context remains active and the RPC returns a clear error

There is no automatic file watcher.

### Auth context swapping

The active auth context will be stored in `ArcSwap<Arc<AuthContext>>`.

Request-handling rule:

- each public request loads the current `Arc<AuthContext>` once at the start of handling
- the request keeps that `Arc` for the duration of the call
- in-flight requests therefore continue safely even if a reload happens concurrently
- new requests see the newly swapped context immediately after a successful store

This gives atomic cutover without interrupting in-flight work.

## Public API Design

### API style

- protobuf API is fully typed
- RPC naming and behavior follow upstream `ytmusicapi` one-to-one
- continuation endpoints remain explicit separate RPCs
- no generic catch-all JSON responses

### Public RPC set

The public API should include RPCs corresponding to the upstream methods available in `ytmusicapi` `v0.1.0`, including:

- `Search`
- `SearchContinuation`
- `GetWatchPlaylist`
- `GetWatchPlaylistContinuation`
- `GetSong`
- `GetLibraryPlaylists`
- `GetLibraryPlaylistsContinuation`
- `GetAccountInfo`
- `GetLibraryArtists`
- `GetLibraryArtistsContinuation`
- `GetLibraryAlbums`
- `GetLibraryAlbumsContinuation`
- `GetLibrarySubscriptions`
- `GetLibrarySubscriptionsContinuation`
- `GetLibraryChannels`
- `GetLibraryChannelsContinuation`
- `GetLibraryPodcasts`
- `GetLibraryPodcastsContinuation`
- `GetLibrarySongs`
- `GetLibrarySongsContinuation`
- `GetLikedSongs`
- `GetLikedSongsContinuation`
- `GetSavedEpisodes`
- `GetSavedEpisodesContinuation`
- `Decipher`

### Typed schema policy

- protobuf messages should preserve the observable shape of upstream models
- enums should be modeled explicitly where stable and known
- stringly typed passthroughs are acceptable only where upstream shape is not represented as a stable Rust enum
- continuation tokens remain typed message fields, not opaque JSON documents

## Decipher Flow

The service does not provide a direct "video ID to playable URL" convenience RPC.

Instead, deciphering is intentionally a two-step workflow:

1. caller invokes `GetSong(video_id)`
2. caller inspects the typed `streaming_data` response and chooses a specific `signatureCipher`
3. caller invokes `Decipher(signature_cipher)`
4. service uses the long-lived `YtCipher` instance to convert that selected cipher into a playable URL
5. service returns the resolved playable URL

This keeps stream selection policy outside the service and makes the decipher boundary explicit.

## Runtime State

### `AuthContext`

`AuthContext` should encapsulate at least:

- validated browser auth headers/config loaded from `browser.json`
- authenticated `YtMusic` client instance
- metadata needed for logging and observability such as a version or reload timestamp

### `YtCipher`

`YtCipher` should be created once during startup and shared across requests.

Responsibilities:

- bootstrap and hold current player solver state
- expose `signature_timestamp` for `GetSong` calls
- resolve caller-selected `signatureCipher` values through `Decipher`

The design assumes `YtCipher` remains independent of `browser.json` rotation.

### Authenticated probe choice

The validation probe used during startup and reload should be a small authenticated upstream call that reliably requires browser auth, such as `GetAccountInfo`.

This keeps startup and reload semantics deterministic:

- success means the service has a currently usable authenticated context
- failure means the service must not start or must reject the reload

## Request Flow

### Standard public RPC

1. tonic handler validates the protobuf request
2. handler loads the current `Arc<AuthContext>` from `ArcSwap`
3. handler calls the matching adapter method
4. adapter calls the matching upstream library function
5. upstream result is translated into typed protobuf output
6. gRPC response is returned

### Reload RPC

1. admin RPC handler reads the configured `browser.json`
2. handler parses and validates the new file
3. handler builds a fresh `AuthContext`
4. handler runs an authenticated probe
5. on success, handler atomically swaps `ArcSwap`
6. on failure, handler leaves the active context unchanged

## Error Handling

### Public RPC status mapping

- invalid client input: `INVALID_ARGUMENT`
- missing auth precondition or unsupported authenticated access path: `FAILED_PRECONDITION`
- auth rejection or permission issue from upstream: `UNAUTHENTICATED` or `PERMISSION_DENIED`
- upstream transport failure or transient network issue: `UNAVAILABLE`
- upstream response parsing or internal mapping failure: `INTERNAL`

### Reload RPC failures

`ReloadBrowserAuth` should return structured errors covering:

- file read failure
- JSON decode failure
- browser auth validation failure
- authenticated probe failure

Reload failure must never partially activate the new auth context.

## Observability and Operations

### Health and reflection

- health service should exist on both listeners
- reflection should be admin-only
- readiness should fail during startup until auth context and `YtCipher` are initialized

### Logging

Logs should include:

- startup auth validation outcome
- reload attempts and outcomes
- active auth context version metadata
- upstream transport/auth failures
- request identifiers where available

Sensitive browser auth content must never be logged.

### Configuration

Minimal runtime configuration:

- public listen address
- admin listen address
- `browser.json` path
- request timeout settings
- logging level / telemetry config

## Container Packaging

Container support must work with both Podman and Docker.

### Build strategy

- multi-stage build
- builder stage compiles the Rust binary and protobuf artifacts
- final runtime image uses a distroless base image

### Runtime image expectations

- contains only the service binary and required runtime assets
- runs as non-root where practical
- expects `browser.json` to be mounted into the container
- exposes public and admin ports explicitly
- avoids shell-based runtime assumptions because the final image is distroless

## Testing Strategy

### Unit tests

- config parsing and validation
- protobuf/domain mapping
- gRPC error translation
- reload state transition behavior

### Integration tests

- mocked upstream HTTP flows for each major `ytmusicapi` RPC family
- `GetSong` plus `Decipher` flow
- continuation method behavior
- authenticated feature handling

### Concurrency and reload tests

- successful reload atomically changes only new requests
- failed reload preserves the prior auth context
- in-flight requests remain safe during concurrent reload

### Container tests

- container starts successfully with valid mounted `browser.json`
- container fails startup when `browser.json` is missing or invalid
- admin reload works through the internal listener in containerized environments

## Implementation Notes

- keep protobuf definitions explicit rather than code-generating schemas directly from Rust structs
- generate and ship the protobuf descriptor set for admin reflection
- prefer small adapter traits around upstream clients to keep tests isolated from tonic transport code
- keep admin and public services in separate tonic service registrations even though they share one process

## Open Decisions Resolved

- service shape: single binary with dual listeners
- auth reload model: explicit admin reload RPC only
- auth swap primitive: `ArcSwap`
- API style: fully typed protobuf
- upstream mapping: one-to-one RPC mapping
- stream selection: caller chooses `signatureCipher`; service only deciphers
- container runtime base: distroless
