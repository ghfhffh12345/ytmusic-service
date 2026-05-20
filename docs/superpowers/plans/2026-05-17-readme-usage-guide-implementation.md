# README Usage Guide Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite `README.md` into a detailed operator-first usage guide and add `docs/API.md` as the separate human-readable gRPC API reference.

**Architecture:** Keep onboarding and operations in `README.md`, and move the full human-written API inventory into `docs/API.md`. All claims must come from repository behavior first, with the `ytmusicapi` browser-auth setup flow sourced from upstream documentation.

**Tech Stack:** Markdown, Rust service source, protobuf definitions, `grpcurl`, `cargo test`, upstream `ytmusicapi` CLI documentation

---

## Planned File Structure

- `README.md`
  - primary onboarding guide for setup, authentication bootstrap, configuration, local/container execution, `grpcurl` examples, reload workflow, and troubleshooting
- `docs/API.md`
  - separate gRPC API reference index summarizing public/admin services and RPC groupings
- `proto/ytmusic/v1/public.proto`
  - source of truth for public service and RPC inventory
- `proto/ytmusic/v1/admin.proto`
  - source of truth for admin service and reload RPC name
- `src/lib.rs`
  - source of truth for listener behavior, health service exposure, and reflection placement
- `src/config.rs`
  - source of truth for required environment variables and startup path validation
- `tests/startup.rs`
  - source of truth for startup failure modes around `browser.json`
- `tests/reload.rs`
  - source of truth for reload behavior and in-memory auth replacement semantics

### Task 1: Rewrite the README setup and runtime sections

**Files:**
- Modify: `README.md`
- Reference: `src/config.rs`
- Reference: `src/lib.rs`
- Test: `README.md`

- [ ] **Step 1: Run a coverage check against the current README headings**

Run:

```bash
rg -n "^## " README.md
```

Expected: only the current minimal headings such as `Required environment`, `Local run`, `Container run`, and `Admin reload` appear, confirming that the detailed setup structure is still missing.

- [ ] **Step 2: Verify the runtime facts that the new setup text must match**

Run:

```bash
sed -n '1,220p' src/config.rs
sed -n '1,220p' src/lib.rs
```

Expected: `src/config.rs` shows the three required environment variables and strict file checks for `YTMUSIC_SERVICE_BROWSER_JSON`; `src/lib.rs` shows the public/admin listener split, health on both listeners, and reflection on the admin listener.

- [ ] **Step 3: Replace the top of `README.md` with the operator setup guide**

```md
# ytmusic-service

`ytmusic-service` is a Rust gRPC wrapper around the upstream `ytmusicapi` and `yt-cipher` integrations. It exposes a public listener for YouTube Music operations and a separate admin listener for operational tasks such as credential reload.

## What the service exposes

- A public gRPC API on `YTMUSIC_SERVICE_PUBLIC_ADDR` for `ytmusic.v1.YtMusicPublic`
- A separate admin gRPC API on `YTMUSIC_SERVICE_ADMIN_ADDR` for `ytmusic.v1.admin.YtMusicAdmin`
- Standard gRPC health checks on both listeners
- gRPC reflection on the admin listener for `grpcurl list` and `grpcurl describe`

Use the admin port for reflection-based discovery. Use the public port for the actual music RPCs.

## Prerequisites

Before starting the service, make sure you have:

- a Rust toolchain if you want to run the service locally with `cargo`
- Docker or Podman if you want to run the container image
- `grpcurl` for health checks and example requests
- `ytmusicapi` installed so you can generate `browser.json`
- Firefox signed in to the YouTube Music account you want the service to use

## Authentication setup with `ytmusicapi-cli`

`ytmusic-service` requires a valid `browser.json` file at startup. This file contains browser-derived YouTube Music authentication material, so treat it like a secret and never commit it to the repository.

The upstream `ytmusicapi` project ships the CLI used to create `browser.json`:

```bash
pip install ytmusicapi
```

Firefox is the recommended browser for this flow because the upstream browser-auth instructions explicitly document how to copy the required request headers from Firefox developer tools.

1. Sign in to the correct account at `https://music.youtube.com`.
2. Open Firefox developer tools and switch to the `Network` tab.
3. Refresh the page or trigger a library/search action so Firefox records authenticated requests.
4. Find a successful `POST` request to `music.youtube.com`, typically a `browse` request.
5. Confirm the request succeeded, then right-click it and copy the request headers.
6. Run the CLI and paste the copied headers when prompted:

```bash
ytmusicapi browser
```

The command writes `browser.json` in your current directory. Keep that file outside version control and store it somewhere stable, such as `./secrets/browser.json` for local runs or a read-only mounted path in containers.

## Configuration

The service reads all configuration from environment variables:

| Variable | Purpose | Example |
| --- | --- | --- |
| `YTMUSIC_SERVICE_PUBLIC_ADDR` | Public gRPC listener for `ytmusic.v1.YtMusicPublic` | `127.0.0.1:50051` |
| `YTMUSIC_SERVICE_ADMIN_ADDR` | Admin gRPC listener for `ytmusic.v1.admin.YtMusicAdmin` | `127.0.0.1:50052` |
| `YTMUSIC_SERVICE_BROWSER_JSON` | Path to the credential file loaded at startup and reload time | `/absolute/path/to/browser.json` |

Startup fails if `YTMUSIC_SERVICE_BROWSER_JSON` is missing, points to a non-file path, contains malformed JSON, or does not pass the startup auth probe.

## Local execution

```bash
export YTMUSIC_SERVICE_PUBLIC_ADDR=127.0.0.1:50051
export YTMUSIC_SERVICE_ADMIN_ADDR=127.0.0.1:50052
export YTMUSIC_SERVICE_BROWSER_JSON="$PWD/secrets/browser.json"

cargo run
```

## Container execution

Build the image:

```bash
podman build -t ytmusic-service .
```

Run the container:

```bash
podman run --rm \
  -p 50051:50051 \
  -p 50052:50052 \
  -e YTMUSIC_SERVICE_PUBLIC_ADDR=0.0.0.0:50051 \
  -e YTMUSIC_SERVICE_ADMIN_ADDR=0.0.0.0:50052 \
  -e YTMUSIC_SERVICE_BROWSER_JSON=/run/secrets/browser.json \
  -v "$PWD/secrets/browser.json:/run/secrets/browser.json:ro" \
  ytmusic-service
```

Replacing the mounted file does not activate new credentials by itself. After updating `browser.json`, call the admin reload RPC.
```

- [ ] **Step 4: Verify the README now contains the setup and runtime guide**

Run:

```bash
rg -n "^## (What the service exposes|Prerequisites|Authentication setup with ytmusicapi-cli|Configuration|Local execution|Container execution)$" README.md
rg -n "pip install ytmusicapi|ytmusicapi browser|YTMUSIC_SERVICE_BROWSER_JSON|reflection on the admin listener" README.md
```

Expected: all six headings are present, and the README now contains the `ytmusicapi` install command, the `ytmusicapi browser` flow, the required env var names, and the admin-reflection note.

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: expand README setup and runtime guide"
```

### Task 2: Add README usage examples, reload workflow, and troubleshooting

**Files:**
- Modify: `README.md`
- Reference: `proto/ytmusic/v1/public.proto`
- Reference: `proto/ytmusic/v1/admin.proto`
- Reference: `tests/startup.rs`
- Reference: `tests/reload.rs`
- Test: `README.md`

- [ ] **Step 1: Check that the operational sections are still missing**

Run:

```bash
rg -n "^## (Practical grpcurl usage|Credential rotation and reload workflow|Troubleshooting|Further reference)$" README.md
```

Expected: no matches yet, or only partial matches if a prior edit drifted from the plan.

- [ ] **Step 2: Verify the exact RPC names and failure modes before writing**

Run:

```bash
sed -n '1,120p' proto/ytmusic/v1/admin.proto
sed -n '1,80p' proto/ytmusic/v1/public.proto
sed -n '1,240p' tests/startup.rs
sed -n '1,260p' tests/reload.rs
```

Expected: the admin service is `ytmusic.v1.admin.YtMusicAdmin` with `ReloadBrowserAuth`; the public service is `ytmusic.v1.YtMusicPublic`; startup and reload tests show the real failure modes that the troubleshooting text must describe.

- [ ] **Step 3: Append the operational README sections**

```md
## Practical grpcurl usage

Use the admin listener for reflection-based discovery:

```bash
grpcurl -plaintext 127.0.0.1:50052 list
grpcurl -plaintext 127.0.0.1:50052 describe ytmusic.v1.YtMusicPublic
```

Check serving status:

```bash
grpcurl -plaintext \
  -d '{"service":"ytmusic.v1.YtMusicPublic"}' \
  127.0.0.1:50052 \
  grpc.health.v1.Health/Check
```

Run a representative public request against the public listener:

```bash
grpcurl -plaintext \
  -d '{"query":"Miles Davis","ignoreSpelling":false}' \
  127.0.0.1:50051 \
  ytmusic.v1.YtMusicPublic/Search
```

Reload credentials after replacing `browser.json`:

```bash
grpcurl -plaintext \
  -d '{}' \
  127.0.0.1:50052 \
  ytmusic.v1.admin.YtMusicAdmin/ReloadBrowserAuth
```

## Credential rotation and reload workflow

1. Generate a fresh `browser.json` with `ytmusicapi browser`, or replace the existing file with an updated version.
2. Confirm the service still has read access to the path referenced by `YTMUSIC_SERVICE_BROWSER_JSON`.
3. Call `ytmusic.v1.admin.YtMusicAdmin/ReloadBrowserAuth` on the admin listener.
4. Wait for a successful response before treating the new credentials as active.

The running process keeps the previous in-memory auth context until reload succeeds. Updating the file without calling the reload RPC is not enough.

## Troubleshooting

- `browser.json` path does not exist: verify `YTMUSIC_SERVICE_BROWSER_JSON` points to a real file before starting the service.
- `browser.json` path is a directory: replace it with a regular file path.
- malformed `browser.json`: regenerate the file with `ytmusicapi browser` and avoid hand-editing the JSON.
- startup probe fails: regenerate credentials from a signed-in browser session and retry.
- address already in use: choose a different public or admin bind address, or stop the conflicting process.
- `grpcurl list` fails on the public port: use the admin port because reflection is only registered there.
- new credentials do not take effect: replace the file and then call `ReloadBrowserAuth`.

## Further reference

- [gRPC API reference](docs/API.md)
- [Public protobuf](proto/ytmusic/v1/public.proto)
- [Admin protobuf](proto/ytmusic/v1/admin.proto)
```

- [ ] **Step 4: Verify the README now contains working discovery and troubleshooting content**

Run:

```bash
rg -n "grpcurl -plaintext 127.0.0.1:50052 list|grpc.health.v1.Health/Check|ytmusic.v1.YtMusicPublic/Search|ytmusic.v1.admin.YtMusicAdmin/ReloadBrowserAuth" README.md
rg -n "reflection is only registered there|malformed browser.json|address already in use|docs/API.md" README.md
```

Expected: the README includes the discovery commands, the representative `Search` request, the reload command, the troubleshooting bullets, and the cross-link to `docs/API.md`.

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: add README grpcurl and troubleshooting guide"
```

### Task 3: Create the separate API reference index

**Files:**
- Create: `docs/API.md`
- Reference: `proto/ytmusic/v1/public.proto`
- Reference: `proto/ytmusic/v1/admin.proto`
- Test: `docs/API.md`

- [ ] **Step 1: Confirm the API reference file does not already exist**

Run:

```bash
test -f docs/API.md && echo "exists" || echo "missing"
```

Expected: `missing`

- [ ] **Step 2: Capture the exact RPC inventory for grouping**

Run:

```bash
rg -n "^  rpc " proto/ytmusic/v1/public.proto proto/ytmusic/v1/admin.proto
```

Expected: all public RPCs and `ReloadBrowserAuth` are listed so the hand-written summary can match the current protobuf surface exactly.

- [ ] **Step 3: Write `docs/API.md` as the human-readable API index**

```md
# ytmusic-service API Reference

`ytmusic-service` exposes two gRPC packages:

- `ytmusic.v1` on the public listener for YouTube Music operations
- `ytmusic.v1.admin` on the admin listener for operational control

## Service names

- Public service: `ytmusic.v1.YtMusicPublic`
- Admin service: `ytmusic.v1.admin.YtMusicAdmin`

Use the admin listener when you want reflection-backed discovery with `grpcurl describe` or `grpcurl list`.

## Public API summary

### Search and discovery

- `Search`: run a YouTube Music search with an optional filter and spelling behavior
- `SearchContinuation`: continue a previous `Search` result set with a continuation token

### Watch playlist and playback metadata

- `GetWatchPlaylist`: fetch watch-playlist metadata for a track, playlist, radio, or shuffle flow
- `GetWatchPlaylistContinuation`: continue a previous watch-playlist response
- `GetSong`: fetch song metadata, including stream formats that may require deciphering
- `Decipher`: translate a `signature_cipher` value into a playable URL

### Library listing families

- `GetLibraryPlaylists` / `GetLibraryPlaylistsContinuation`
- `GetLibraryArtists` / `GetLibraryArtistsContinuation`
- `GetLibraryAlbums` / `GetLibraryAlbumsContinuation`
- `GetLibrarySubscriptions` / `GetLibrarySubscriptionsContinuation`
- `GetLibraryChannels` / `GetLibraryChannelsContinuation`
- `GetLibraryPodcasts` / `GetLibraryPodcastsContinuation`
- `GetLibrarySongs` / `GetLibrarySongsContinuation`
- `GetLikedSongs` / `GetLikedSongsContinuation`
- `GetSavedEpisodes` / `GetSavedEpisodesContinuation`

Each continuation RPC consumes the token returned by its corresponding listing call.

### Account information

- `GetAccountInfo`: return account/profile information for the authenticated YouTube Music session

## Admin API summary

- `ReloadBrowserAuth`: reload `browser.json` from the configured path and swap the in-memory auth context if validation succeeds

## Proto sources

- [Public API proto](../proto/ytmusic/v1/public.proto)
- [Admin API proto](../proto/ytmusic/v1/admin.proto)
```

- [ ] **Step 4: Verify the API reference covers the current gRPC surface**

Run:

```bash
rg -n "ytmusic\.v1\.YtMusicPublic|ytmusic\.v1\.admin\.YtMusicAdmin|SearchContinuation|GetWatchPlaylist|GetLibrarySongs|GetSavedEpisodes|ReloadBrowserAuth|proto" docs/API.md
```

Expected: the file contains both service names, representative grouped public RPCs, the reload RPC, and links back to the protobuf sources.

- [ ] **Step 5: Commit**

```bash
git add -f docs/API.md
git commit -m "docs: add ytmusic-service API reference"
```

### Task 4: Run final documentation verification

**Files:**
- Modify: `docs/API.md`
- Test: `README.md`
- Test: `docs/API.md`
- Test: `cargo test`

- [ ] **Step 1: Check whether the API reference still needs a backlink to the README**

Run:

```bash
rg -n "For setup, authentication bootstrap, runtime examples, and troubleshooting, start with \[README.md\]\(\.\./README.md\)\." docs/API.md
```

Expected: no matches yet, because `docs/API.md` does not include the backlink introduced in this final pass.

- [ ] **Step 2: Add the backlink note at the top of `docs/API.md`**

```md
# ytmusic-service API Reference

For setup, authentication bootstrap, runtime examples, and troubleshooting, start with [README.md](../README.md).

`ytmusic-service` exposes two gRPC packages:
```

- [ ] **Step 3: Run the final documentation verification sweep**

Run:

```bash
rg -n "^## " README.md
rg -n "pip install ytmusicapi|ytmusicapi browser|grpcurl|ReloadBrowserAuth|docs/API.md" README.md docs/API.md
rg -n "For setup, authentication bootstrap, runtime examples, and troubleshooting, start with \[README.md\]\(\.\./README.md\)\." docs/API.md
```

Expected: the README section structure is complete, the `ytmusicapi` install and browser-auth commands are present, the `grpcurl` examples are present, and `docs/API.md` now links readers back to the README for setup and operations.

- [ ] **Step 4: Run the repository tests after the documentation update**

Run:

```bash
cargo test
```

Expected: PASS. The docs change does not alter service behavior, so the repository test suite should remain green.

- [ ] **Step 5: Commit**

```bash
git add docs/API.md
git commit -m "docs: add final docs cross-link and verification pass"
```
