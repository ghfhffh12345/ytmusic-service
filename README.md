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
