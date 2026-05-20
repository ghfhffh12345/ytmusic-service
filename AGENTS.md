Use Serena for project context and Context7 for library/docs context.

Repository overview:
- `ytmusic-service` is a Rust 2024 Cargo workspace for a gRPC YouTube Music service and its public protobuf contract.
- The workspace has two member crates under `crates/`:
  - `crates/ytmusic-service`: the runnable server crate. It owns runtime config, auth loading/reload, adapter code, gRPC service implementations, and integration tests.
  - `crates/ytmusic-service-proto`: the public proto crate. It owns the `.proto` sources, `build.rs`, generated tonic/prost types, generated client/server modules, descriptor exports, and a smoke test for those exports.
- Workspace-level files:
  - `Cargo.toml`: virtual workspace manifest with shared dependency versions.
  - `Containerfile`: builds the `ytmusic-service` package from the workspace.
  - `README.md`: operator/user quickstart and usage notes.
  - `docs/API.md`: API and proto reference.
  - `docs/superpowers/`: design specs and implementation plans from prior agent work.
