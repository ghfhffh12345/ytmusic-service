# ytmusic-service

`ytmusic-service` is a Rust gRPC wrapper around upstream `ytmusicapi` and `yt-cipher`, with separate public and admin listeners.

## Quickstart with Podman

If you already have Podman and a valid `browser.json`, this is the fastest way to run the service.

If you still need to create `browser.json`, follow [Authentication setup with ytmusicapi-cli](#authentication-setup-with-ytmusicapi-cli).

Podman will pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically if it is not already present locally.

```bash
podman run --rm \
  -p 50051:50051 \
  -p 50052:50052 \
  -e YTMUSIC_SERVICE_PUBLIC_ADDR=0.0.0.0:50051 \
  -e YTMUSIC_SERVICE_ADMIN_ADDR=0.0.0.0:50052 \
  -e YTMUSIC_SERVICE_BROWSER_JSON=/run/secrets/browser.json \
  -v "$PWD/browser.json:/run/secrets/browser.json:ro,Z" \
  ghcr.io/ghfhffh12345/ytmusic-service:latest
```

Replacing the mounted file does not activate new credentials until the admin reload RPC is called.

## What the service exposes

- Public gRPC API on `YTMUSIC_SERVICE_PUBLIC_ADDR` for `ytmusic.v1.YtMusicPublic`
- Separate admin gRPC API on `YTMUSIC_SERVICE_ADMIN_ADDR` for `ytmusic.v1.admin.YtMusicAdmin`
- Standard gRPC health checks on both listeners
- gRPC reflection on the admin listener for `grpcurl list` and `grpcurl describe`

Use the admin port for reflection-based discovery, and the public port for actual music RPCs.

## Prerequisites

- Rust toolchain for local runs
- Docker or Podman for container runs
- `grpcurl` for health checks and example requests
- `ytmusicapi` installed so you can generate `browser.json`
- A valid `browser.json` for the target YouTube Music account

## Authentication setup with ytmusicapi-cli

`browser.json` is required at startup and contains sensitive authenticated browser headers. Treat it like a secret.

Install the upstream CLI:

```bash
pip install ytmusicapi
```

The service only requires a valid `browser.json`. Firefox is the recommended flow here because the upstream `ytmusicapi` browser-auth instructions explicitly describe the Firefox header capture flow.

1. Sign in to `https://music.youtube.com` in Firefox with the account this service should use.
2. Open Firefox Developer Tools and switch to the Network tab.
3. Trigger authenticated traffic in YouTube Music so the Network tab captures requests.
4. Find a successful `POST` request such as `browse`.
5. Copy the request headers from that request.
6. Run `ytmusicapi browser`.
7. Paste the copied headers when prompted.

The command writes `browser.json` in the current directory. Keep it out of version control. The source-based local run path below assumes it remains at `./browser.json`; if you move it elsewhere, update `YTMUSIC_SERVICE_BROWSER_JSON` to match.

## Configuration

| Variable | Purpose | Example |
| --- | --- | --- |
| `YTMUSIC_SERVICE_PUBLIC_ADDR` | Bind address for the public gRPC listener serving `ytmusic.v1.YtMusicPublic` and health checks | `127.0.0.1:50051` |
| `YTMUSIC_SERVICE_ADMIN_ADDR` | Bind address for the admin gRPC listener serving `ytmusic.v1.admin.YtMusicAdmin`, health checks, and reflection | `127.0.0.1:50052` |
| `YTMUSIC_SERVICE_BROWSER_JSON` | Filesystem path to the `browser.json` credentials file loaded at startup | `/absolute/path/to/browser.json` |

Startup fails if the browser json path is missing, points to something other than a file, contains malformed JSON, contains invalid or otherwise unusable auth data, or fails the startup auth probe.

## Run locally from source

Use this path if you want to run the service from this repository instead of the published container image.

```bash
git clone https://github.com/ghfhffh12345/ytmusic-service.git
cd ytmusic-service

export YTMUSIC_SERVICE_PUBLIC_ADDR=127.0.0.1:50051
export YTMUSIC_SERVICE_ADMIN_ADDR=127.0.0.1:50052
export YTMUSIC_SERVICE_BROWSER_JSON="$PWD/browser.json"

cargo run
```

## Practical grpcurl usage

The admin listener is the endpoint for reflection-based discovery, so use it for `grpcurl list` and `grpcurl describe`.

List reflected services on the admin listener:

```bash
grpcurl -plaintext 127.0.0.1:50052 list
```

Describe the public API service from the admin listener:

```bash
grpcurl -plaintext 127.0.0.1:50052 describe ytmusic.v1.YtMusicPublic
```

Check the public service health on the public listener:

```bash
grpcurl -plaintext \
  -d '{"service":"ytmusic.v1.YtMusicPublic"}' \
  127.0.0.1:50051 \
  grpc.health.v1.Health/Check
```

Send a representative search request to the public listener:

```bash
grpcurl -plaintext \
  -d '{"query":"Miles Davis","ignoreSpelling":false}' \
  127.0.0.1:50051 \
  ytmusic.v1.YtMusicPublic/Search
```

Reload browser credentials through the admin listener:

```bash
grpcurl -plaintext \
  -d '{}' \
  127.0.0.1:50052 \
  ytmusic.v1.admin.YtMusicAdmin/ReloadBrowserAuth
```

## Credential rotation and reload workflow

1. Generate or replace `browser.json` with the new authenticated browser headers.
2. Confirm the service process can read the configured `YTMUSIC_SERVICE_BROWSER_JSON` path and that the path still resolves to a regular file.
3. Call the reload RPC on the admin listener.
4. Wait for the reload RPC to succeed before treating the new credentials as active.

The service keeps the prior in-memory auth context until reload succeeds, and replacing the file alone is not enough to activate new credentials.

## Troubleshooting

- Missing `browser.json`: startup fails if `YTMUSIC_SERVICE_BROWSER_JSON` points to a path that does not exist.
- `browser.json` path is a directory: startup fails if the configured path is not a regular file.
- Malformed `browser.json`: regenerate `browser.json` with `ytmusicapi browser`, replace the configured file, and retry startup or call reload again.
- Startup probe fails: if startup validation rejects the auth, regenerate `browser.json` with `ytmusicapi browser`, replace the configured file, and retry startup or call reload again.
- address already in use: if either listener cannot bind its configured socket address, free the port or choose different values for `YTMUSIC_SERVICE_PUBLIC_ADDR` and `YTMUSIC_SERVICE_ADMIN_ADDR`.
- `grpcurl list` fails on the public port: reflection is only registered there on the admin port, so use `127.0.0.1:50052` for discovery commands.
- Credential file replaced without reload: the running process keeps using the previous in-memory auth context until `ReloadBrowserAuth` succeeds.

## Further reference

- [docs/API.md](docs/API.md)
- [proto/ytmusic/v1/public.proto](proto/ytmusic/v1/public.proto)
- [proto/ytmusic/v1/admin.proto](proto/ytmusic/v1/admin.proto)
