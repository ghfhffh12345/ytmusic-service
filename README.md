# ytmusic-service

`ytmusic-service` is a single-listener gRPC service for authenticated YouTube Music access.

## What it provides

- One gRPC listener on `YTMUSIC_SERVICE_ADDR`
- Browser-authenticated startup from `YTMUSIC_SERVICE_BROWSER_JSON`
- Optional positive per-RPC timeout from `YTMUSIC_SERVICE_RPC_TIMEOUT_MS`
- Reflected and health-checked v2 service surface on the same port

The exposed services are:

- `ytmusic.v2.YtMusic`
- `ytmusic.v2.YtCipher`
- `ytmusic.v2.ServiceStatus`
- `grpc.health.v1.Health`
- gRPC reflection

## Before you start

You will need:

- Docker for container runs and local image builds
- `grpcurl` if you want to inspect services or send test requests
- A valid `browser.json` for the YouTube Music account the service should use

Generate `browser.json` by following the guide in [`ghfhffh12345/ytmusicapi`](https://github.com/ghfhffh12345/ytmusicapi#generate-browserjson-with-ytmusicapi-cli). Treat it as a secret.

Startup fails if the configured `browser.json` path is missing, is not a regular file, contains malformed JSON, contains unusable auth data, or fails the startup auth probe.

## Configuration

| Variable | Purpose | Example |
| --- | --- | --- |
| `YTMUSIC_SERVICE_ADDR` | Bind address for all gRPC traffic, health checks, and reflection | `127.0.0.1:50051` |
| `YTMUSIC_SERVICE_BROWSER_JSON` | Filesystem path to the `browser.json` credentials file | `/absolute/path/to/browser.json` |
| `YTMUSIC_SERVICE_RPC_TIMEOUT_MS` | Optional positive server-side timeout in milliseconds applied to each RPC | `15000` |

## Run with Docker

If you already have Docker and a valid `browser.json`, this is the fastest way to start the service. Docker will pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically if it is not already present locally.

```bash
docker run --rm \
  -p 50051:50051 \
  -e YTMUSIC_SERVICE_ADDR=0.0.0.0:50051 \
  -e YTMUSIC_SERVICE_BROWSER_JSON=/run/secrets/browser.json \
  -v "$PWD/browser.json:/run/secrets/browser.json:ro" \
  ghcr.io/ghfhffh12345/ytmusic-service:latest
```

## Publish images

GitHub Actions publishes container images automatically on GitHub-hosted runners when you publish a GitHub Release for an exact `vX.Y.Z` tag. The release workflow requires the ignored live smoke test to pass before image publication starts.

The live release gate requires these GitHub Actions secrets:

- `YTMUSIC_SERVICE_LIVE_BROWSER_JSON`: the raw `browser.json` contents
- `YTMUSIC_SERVICE_LIVE_VIDEO_ID`: a known-good song video ID for `GetSong`
- `YTMUSIC_SERVICE_LIVE_QUERY`: optional search query override; defaults to `Miles Davis`

During the workflow, the browser JSON secret is written to a temporary file, mounted into a locally built smoke-test container, and passed through the same `YTMUSIC_SERVICE_ADDR` and `YTMUSIC_SERVICE_BROWSER_JSON` env surface the image uses at runtime. The ignored smoke test connects through `YTMUSIC_SERVICE_LIVE_ENDPOINT` to validate that container before any manifest publication begins.

Create and publish a GitHub Release for the exact tag, for example `v0.1.1`. Publishing the release, not pushing the tag alone, starts the image workflow.

A successful workflow run publishes these GHCR tags:

- `ghcr.io/ghfhffh12345/ytmusic-service:0.1.1`
- `ghcr.io/ghfhffh12345/ytmusic-service:0.1`
- `ghcr.io/ghfhffh12345/ytmusic-service:0`
- `ghcr.io/ghfhffh12345/ytmusic-service:latest`

The `latest`, major, and minor tags are updated by every published release that uses an exact `vX.Y.Z` tag.
GitHub prereleases published on an exact `vX.Y.Z` tag follow the same image-tag promotion rule.

The published image is a multi-architecture manifest for:

- `linux/amd64`
- `linux/arm64`

The workflow rejects tags such as `v0.1`, `v0`, and `v0.1.1-rc1`. Use exact `vX.Y.Z` tags only for published releases.

## Run from source

Use this path if you want to run the service from this repository instead of the published container image.

```bash
git clone https://github.com/ghfhffh12345/ytmusic-service.git
cd ytmusic-service

export YTMUSIC_SERVICE_ADDR=127.0.0.1:50051
export YTMUSIC_SERVICE_BROWSER_JSON="$PWD/browser.json"
# export YTMUSIC_SERVICE_RPC_TIMEOUT_MS=15000  # must be greater than 0

cargo run -p ytmusic-service
```

The source-based example assumes `browser.json` is available at `./browser.json`. If you store it elsewhere, update `YTMUSIC_SERVICE_BROWSER_JSON` to match.

## Rust consumers

Rust callers who want an ergonomic grouped client should use `ytmusic-service-client`.
Rust callers who want the raw generated gRPC contract should use `ytmusic-service-proto`.

## Verify and use the service

List reflected services on the service listener:

```bash
grpcurl -plaintext 127.0.0.1:50051 list
```

Describe the music API:

```bash
grpcurl -plaintext 127.0.0.1:50051 describe ytmusic.v2.YtMusic
```

Check service health:

```bash
grpcurl -plaintext \
  -d '{"service":"ytmusic.v2.YtMusic"}' \
  127.0.0.1:50051 \
  grpc.health.v1.Health/Check
```

This health response confirms the process is serving that gRPC service on startup. It does not continuously probe upstream YouTube Music dependencies.

Send a representative search request:

```bash
grpcurl -plaintext \
  -d '{"query":"Miles Davis","filter":"SEARCH_FILTER_SONGS","ignoreSpelling":false}' \
  127.0.0.1:50051 \
  ytmusic.v2.YtMusic/Search
```

Inspect service status:

```bash
grpcurl -plaintext \
  -d '{}' \
  127.0.0.1:50051 \
  ytmusic.v2.ServiceStatus/GetStatus
```

`GetStatus` reports process-level startup metadata and the service's current in-process readiness view. It is not a continuous upstream liveness guarantee.

## Rotate browser credentials

The service loads `browser.json` at startup. To rotate credentials, replace the file and restart the process with the same `YTMUSIC_SERVICE_BROWSER_JSON` path.

## Troubleshooting

- Missing `browser.json`: startup fails if `YTMUSIC_SERVICE_BROWSER_JSON` points to a path that does not exist.
- `browser.json` path is a directory: startup fails if the configured path is not a regular file.
- Malformed `browser.json` or failed startup probe: regenerate the file using the [`ytmusicapi` guide](https://github.com/ghfhffh12345/ytmusicapi#generate-browserjson-with-ytmusicapi-cli), replace the configured file, and restart the service.
- Invalid `YTMUSIC_SERVICE_RPC_TIMEOUT_MS`: if set, it must be a positive integer greater than `0`.
- Address already in use: free the port or choose a different `YTMUSIC_SERVICE_ADDR`.
- `grpcurl list` fails: reflection is served on the same listener as the gRPC APIs, so target `127.0.0.1:50051`.

## Further reference

- [docs/API.md](docs/API.md)
- [crates/ytmusic-service-proto/proto/ytmusic/v2/music.proto](crates/ytmusic-service-proto/proto/ytmusic/v2/music.proto)
- [crates/ytmusic-service-proto/proto/ytmusic/v2/cipher.proto](crates/ytmusic-service-proto/proto/ytmusic/v2/cipher.proto)
- [crates/ytmusic-service-proto/proto/ytmusic/v2/status.proto](crates/ytmusic-service-proto/proto/ytmusic/v2/status.proto)
- [ghfhffh12345/ytmusicapi](https://github.com/ghfhffh12345/ytmusicapi)
