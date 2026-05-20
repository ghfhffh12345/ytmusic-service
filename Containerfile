FROM docker.io/library/rust:1.88-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p ytmusic-service

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/ytmusic-service /app/ytmusic-service
USER nonroot:nonroot
EXPOSE 50051 50052
ENTRYPOINT ["/app/ytmusic-service"]
