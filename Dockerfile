# syntax=docker/dockerfile:1.7
FROM --platform=$BUILDPLATFORM docker.io/library/rust:1.88-bookworm AS builder
ARG BUILDARCH
ARG TARGETARCH

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    set -eux; \
    case "${TARGETARCH}" in \
      amd64) \
        rust_target="x86_64-unknown-linux-gnu"; \
        if [ "${BUILDARCH}" = "amd64" ]; then \
          linker=""; \
          cross_packages=""; \
        else \
          linker="x86_64-linux-gnu-gcc"; \
          cross_packages="gcc-x86-64-linux-gnu libc6-dev-amd64-cross"; \
        fi \
        ;; \
      arm64) \
        rust_target="aarch64-unknown-linux-gnu"; \
        if [ "${BUILDARCH}" = "arm64" ]; then \
          linker=""; \
          cross_packages=""; \
        else \
          linker="aarch64-linux-gnu-gcc"; \
          cross_packages="gcc-aarch64-linux-gnu libc6-dev-arm64-cross"; \
        fi \
        ;; \
      *) \
        echo "Unsupported TARGETARCH: ${TARGETARCH}" >&2; \
        exit 1; \
        ;; \
    esac; \
    if [ -n "${cross_packages}" ]; then \
      apt-get update; \
      apt-get install -y --no-install-recommends ${cross_packages}; \
      rm -rf /var/lib/apt/lists/*; \
    fi; \
    rustup target add "${rust_target}"; \
    if [ -n "${linker}" ]; then \
      linker_env_var="CARGO_TARGET_$(printf '%s' "${rust_target}" | tr '[:lower:]-' '[:upper:]_')_LINKER"; \
      export "${linker_env_var}=${linker}"; \
    fi; \
    cargo build --locked --release --target "${rust_target}" -p ytmusic-service; \
    cp "target/${rust_target}/release/ytmusic-service" /tmp/ytmusic-service

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12
LABEL org.opencontainers.image.source="https://github.com/ghfhffh12345/ytmusic-service"
WORKDIR /app
COPY --from=builder /tmp/ytmusic-service /app/ytmusic-service
USER nonroot:nonroot
EXPOSE 50051 50052
ENTRYPOINT ["/app/ytmusic-service"]
