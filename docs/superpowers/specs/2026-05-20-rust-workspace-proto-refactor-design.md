# Rust Workspace And Public Proto Crate Refactor Design

Date: 2026-05-20

## Goal

Refactor the current single-crate repository into a Rust workspace with a clean separation between:

- `ytmusic-service` as the runnable server crate
- `ytmusic-service-proto` as the public crate that owns protobuf generation and exports the gRPC contract

The refactor should optimize for a clean structure instead of Rust import backward compatibility.

## Scope

This design covers:

- converting the repository root into a virtual Cargo workspace
- moving the existing server package into a workspace member
- introducing a public `ytmusic-service-proto` crate
- moving protobuf generation and descriptor ownership into `ytmusic-service-proto`
- rewiring the server crate and tests to consume generated code from `ytmusic-service-proto`
- updating container and documentation paths for the workspace layout

## Non-Goals

- preserving `ytmusic_service::proto::...` imports
- creating a separate `ytmusic-service-client` crate
- adding a handwritten Rust SDK or convenience client layer
- changing the gRPC API shape, RPC list, runtime auth model, or admin/public listener model
- committing generated Rust protobuf output to the repository

## Chosen Direction

Use a virtual workspace root with two member crates:

- `crates/ytmusic-service`
- `crates/ytmusic-service-proto`

`ytmusic-service-proto` is the public dependency for Rust consumers. It exports:

- generated `prost` message types
- generated `tonic` client modules
- generated `tonic` server modules
- file descriptor set bytes needed for server reflection

`ytmusic-service` depends on `ytmusic-service-proto` and owns all runtime behavior:

- environment/config parsing
- browser auth loading and reload
- app state
- upstream `ytmusicapi` and `yt-cipher` adapters
- tonic server wiring
- reflection and health registration
- integration tests
- binary packaging

There will be no compatibility re-export layer from `ytmusic-service`.

## Alternatives Considered

### 1. Virtual workspace with separate proto crate

This is the selected approach.

Why it wins:

- the server depends on the wire contract instead of owning it
- the public crate identity stays focused on the protobuf contract
- future workspace growth stays straightforward
- Cargo metadata and build responsibilities become easier to reason about

### 2. Root package remains `ytmusic-service` and also becomes the workspace root

This would require less file movement, but it leaves the root doing two jobs at once:

- repository orchestration
- product package ownership

That structure is workable but less clean for a multi-crate repository.

### 3. Separate proto crate with committed generated Rust

This avoids build-time generation for consumers, but it adds generated-file churn and drift risk. The repository already uses a protobuf build step, so moving that build step into the proto crate is the cleaner continuation.

## Repository Layout

The repository should become:

```text
Cargo.toml
Cargo.lock
Containerfile
README.md
docs/
crates/
  ytmusic-service/
    Cargo.toml
    src/
    tests/
  ytmusic-service-proto/
    Cargo.toml
    build.rs
    proto/
      ytmusic/v1/public.proto
      ytmusic/v1/admin.proto
    src/
      lib.rs
```

The root `Cargo.toml` should be a virtual workspace manifest with:

- `[workspace]`
- `members = ["crates/ytmusic-service", "crates/ytmusic-service-proto"]`
- `resolver = "3"`

The root may also use `[workspace.dependencies]` and `[workspace.package]` where that reduces repeated metadata cleanly, but the design does not require aggressive centralization.

## Crate Responsibilities

### `ytmusic-service-proto`

This crate becomes the single owner of:

- `.proto` source files
- `build.rs` protobuf compilation
- generated `tonic` client and server code
- descriptor-set generation for reflection

It should expose generated modules directly as the canonical import path for Rust consumers. The crate should remain low-level and generated-code-oriented. It should not add service-specific endpoint helpers, config loaders, auth abstractions, or error mapping policy.

### `ytmusic-service`

This crate remains the runnable service and keeps the package name `ytmusic-service`.

It should contain:

- `src/main.rs` as the thin binary entrypoint
- `src/lib.rs` and the existing runtime modules
- auth, config, state, adapters, and server implementations
- integration tests that exercise runtime behavior

It should stop compiling protobufs directly and instead depend on `ytmusic-service-proto` for:

- generated request and response types
- generated server traits
- generated server constructors
- reflection descriptor constants

## Data Flow And Boundaries

The runtime behavior does not change conceptually:

1. `ytmusic-service` loads configuration and browser auth.
2. `ytmusic-service` constructs shared runtime state.
3. `PublicService` and `AdminService` implement the generated server traits from `ytmusic-service-proto`.
4. The tonic server registers health services, reflection, and the generated gRPC services.
5. External Rust consumers depend directly on `ytmusic-service-proto` and use the generated clients and message types.

The key design rule is:

- wire-contract code lives in `ytmusic-service-proto`
- runtime/policy code lives in `ytmusic-service`

Neither crate should blur that boundary.

## Build And Generation Model

`ytmusic-service-proto` should own the current protobuf build logic:

- vendored `protoc`
- `tonic-build`
- descriptor generation for both public and admin protos

The generated module surface should include both public and admin packages under the existing protobuf namespace. The server crate should no longer contain a local `proto` module or local protobuf `build.rs`.

This means:

- `build.rs` moves from the current root package into `crates/ytmusic-service-proto/`
- `proto/ytmusic/v1/*.proto` move into `crates/ytmusic-service-proto/proto/`
- `src/proto.rs` is removed from `ytmusic-service`

## Server Integration Changes

The server crate should be updated to import generated code from `ytmusic_service_proto` directly.

This includes:

- service trait implementations in `servers/public.rs` and `servers/admin.rs`
- reflection descriptor registration in `lib.rs`
- tests that currently reference `ytmusic_service::proto::...`

Because the project explicitly favors a clean break, the server crate should not re-export proto types for compatibility.

## Error Handling

Error handling remains a server concern.

`ytmusic-service-proto` should not own:

- runtime validation
- gRPC status translation
- environment/config errors
- browser auth semantics
- upstream library error normalization

Those behaviors belong in `ytmusic-service`, because they are properties of how this service operates, not properties of the protobuf contract itself.

## Testing Strategy

### `ytmusic-service-proto`

This crate mainly needs build-level validation:

- protobuf compilation succeeds
- generated client/server surfaces compile
- descriptor bytes remain available to downstream users

No rich runtime integration suite is required in the proto crate unless a concrete regression target appears later.

### `ytmusic-service`

The current runtime-focused tests stay with the server crate because they verify:

- config validation
- browser auth loading behavior
- reload semantics
- status-code mapping
- tonic handler behavior

These tests should be updated to import generated types from `ytmusic_service_proto` rather than from the server crate.

## Packaging And Developer Workflow

The root `Containerfile` remains at the repository root, but it must become workspace-aware.

It should:

- copy the workspace root manifest and lockfile
- copy both member crate directories
- build `cargo build --release -p ytmusic-service`
- copy only the `ytmusic-service` release binary into the runtime image

Repository commands should likewise become workspace-aware:

- `cargo build -p ytmusic-service`
- `cargo run -p ytmusic-service`
- `cargo test --workspace`

Docs should describe `ytmusic-service-proto` as the Rust crate that direct consumers depend on when they want the generated API surface.

## Migration Rules

The implementation should follow these rules:

- keep the public gRPC API unchanged
- keep the server package name `ytmusic-service`
- do not add a `ytmusic-service-client` crate
- do not preserve old Rust import paths
- do not duplicate protobuf generation in more than one crate
- do not move runtime policy into the proto crate

## Success Criteria

The refactor is complete when:

1. the repository builds as a Cargo workspace
2. `ytmusic-service-proto` is a standalone public crate that exports generated messages plus generated tonic client and server modules
3. `ytmusic-service` compiles and runs while depending on `ytmusic-service-proto`
4. existing runtime tests pass after updating imports and paths
5. container builds target the workspace layout correctly
6. root docs and commands reflect the new workspace structure
