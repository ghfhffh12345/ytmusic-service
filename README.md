# ytmusic-service

`ytmusic-service` is a gRPC service layer that provides various YouTube Music features through separate public and admin listeners.

## What it provides

- Public gRPC API on `YTMUSIC_SERVICE_PUBLIC_ADDR` for `ytmusic.v1.YtMusicPublic`
- Separate admin gRPC API on `YTMUSIC_SERVICE_ADMIN_ADDR` for `ytmusic.v1.admin.YtMusicAdmin`
- Standard gRPC health checks on both listeners
- gRPC reflection on the admin listener for `grpcurl list` and `grpcurl describe`

Use the admin port for reflection and admin RPCs. Use the public port for music RPCs.

## Before you start

You will need:

- Docker or Podman for container runs, or the Rust toolchain for local runs
- `grpcurl` if you want to inspect services or send test requests
- A valid `browser.json` for the YouTube Music account the service should use

Generate `browser.json` by following the guide in [`ghfhffh12345/ytmusicapi`](https://github.com/ghfhffh12345/ytmusicapi#generate-browserjson-with-ytmusicapi-cli). Treat it as a secret.

The service loads browser authentication from `YTMUSIC_SERVICE_BROWSER_JSON` at startup. Replacing the file on disk does not activate new credentials until `ReloadBrowserAuth` succeeds.

## Configuration

| Variable | Purpose | Example |
| --- | --- | --- |
| `YTMUSIC_SERVICE_PUBLIC_ADDR` | Bind address for the public gRPC listener and its health checks | `127.0.0.1:50051` |
| `YTMUSIC_SERVICE_ADMIN_ADDR` | Bind address for the admin gRPC listener, admin health checks, and reflection | `127.0.0.1:50052` |
| `YTMUSIC_SERVICE_BROWSER_JSON` | Filesystem path to the `browser.json` credentials file | `/absolute/path/to/browser.json` |

Startup fails if the configured `browser.json` path is missing, is not a regular file, contains malformed JSON, contains unusable auth data, or fails the startup auth probe.

## Run with Podman

If you already have Podman and a valid `browser.json`, this is the fastest way to start the service. Podman will pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically if it is not already present locally.

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

## Run from source

Use this path if you want to run the service from this repository instead of the published container image.

```bash
git clone https://github.com/ghfhffh12345/ytmusic-service.git
cd ytmusic-service

export YTMUSIC_SERVICE_PUBLIC_ADDR=127.0.0.1:50051
export YTMUSIC_SERVICE_ADMIN_ADDR=127.0.0.1:50052
export YTMUSIC_SERVICE_BROWSER_JSON="$PWD/browser.json"

cargo run
```

The source-based example assumes `browser.json` is available at `./browser.json`. If you store it elsewhere, update `YTMUSIC_SERVICE_BROWSER_JSON` to match.

## Verify and use the service

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

## Rotate browser credentials

1. Generate or replace `browser.json` by following the guide in [`ghfhffh12345/ytmusicapi`](https://github.com/ghfhffh12345/ytmusicapi#generate-browserjson-with-ytmusicapi-cli).
2. Confirm the service process can read the configured `YTMUSIC_SERVICE_BROWSER_JSON` path and that the path still resolves to a regular file.
3. Call `ReloadBrowserAuth` on the admin listener.
4. Wait for the reload RPC to succeed before treating the new credentials as active.

## Troubleshooting

- Missing `browser.json`: startup fails if `YTMUSIC_SERVICE_BROWSER_JSON` points to a path that does not exist.
- `browser.json` path is a directory: startup fails if the configured path is not a regular file.
- Malformed `browser.json` or failed startup probe: regenerate the file using the [`ytmusicapi` guide](https://github.com/ghfhffh12345/ytmusicapi#generate-browserjson-with-ytmusicapi-cli), replace the configured file, and retry startup or call reload again.
- Address already in use: free the port or choose different values for `YTMUSIC_SERVICE_PUBLIC_ADDR` and `YTMUSIC_SERVICE_ADMIN_ADDR`.
- `grpcurl list` fails on the public port: reflection is only registered on the admin port, so use `127.0.0.1:50052`.
- Credential file replaced without reload: the running process keeps using the previous in-memory auth context until `ReloadBrowserAuth` succeeds.

## Further reference

- [docs/API.md](docs/API.md)
- [proto/ytmusic/v1/public.proto](proto/ytmusic/v1/public.proto)
- [proto/ytmusic/v1/admin.proto](proto/ytmusic/v1/admin.proto)
- [ghfhffh12345/ytmusicapi](https://github.com/ghfhffh12345/ytmusicapi)
