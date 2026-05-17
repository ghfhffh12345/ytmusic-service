# README Usage Guide Design

## Summary

Refresh the project documentation so `README.md` becomes a detailed, operator-first usage guide for `ytmusic-service`, while `docs/API.md` becomes the separate human-written API reference index. The README should help a new user obtain `browser.json`, run the service locally or in a container, validate the listeners with `grpcurl`, reload credentials safely, and troubleshoot common startup or runtime failures.

## Goals

- Make first-time setup practical for both service operators and API consumers.
- Keep the README executable, not just descriptive, by including concrete commands and request examples.
- Document the `browser.json` bootstrap flow using `ytmusicapi-cli`, with Firefox recommended for header capture.
- Separate onboarding and operations guidance from the full RPC catalog so the README stays focused.
- Point users to the protobuf sources and a dedicated API reference page for deeper exploration.

## Non-Goals

- Do not change the service implementation, protobuf schema, or runtime behavior.
- Do not duplicate every request/response field from the protobuf files into hand-written docs.
- Do not document unsupported deployment patterns such as automatic secret rotation or orchestration-specific manifests.

## Current State

The existing `README.md` only lists required environment variables, a minimal local run command, a container run command, and a brief note about calling `ReloadBrowserAuth`. It does not explain how to create `browser.json`, how the public and admin listeners differ, how to inspect the service with `grpcurl`, or how to diagnose common failures.

The service surface relevant to the docs is:

- Public gRPC listener configured by `YTMUSIC_SERVICE_PUBLIC_ADDR`
- Admin gRPC listener configured by `YTMUSIC_SERVICE_ADMIN_ADDR`
- Startup requirement for a valid `browser.json` file configured by `YTMUSIC_SERVICE_BROWSER_JSON`
- gRPC health service on both listeners
- gRPC reflection enabled on the admin listener
- Public service `ytmusic.v1.YtMusicPublic`
- Admin service `ytmusic.v1.admin.YtMusicAdmin` with `ReloadBrowserAuth`

## Audience

### Primary audience

- Operators who need to bootstrap authentication, configure the process, and run it reliably

### Secondary audience

- API consumers who need enough examples to connect, discover services, and make representative gRPC calls

## Deliverables

### `README.md`

Rewrite as a task-oriented usage guide with the following sections:

1. Project overview
2. What the service exposes
3. Prerequisites
4. Authentication setup with `ytmusicapi-cli`
5. Configuration reference
6. Local execution
7. Container execution
8. Practical `grpcurl` usage
9. Credential rotation and reload workflow
10. Troubleshooting
11. Further reference

### `docs/API.md`

Create a separate API reference page that summarizes the public and admin gRPC surface without repeating the full protobuf definitions.

## README Content Design

### 1. Project overview

Open with a short explanation that `ytmusic-service` is a Rust gRPC wrapper around the upstream YouTube Music integration crates. State that the service separates public API traffic from admin operations by using two listeners.

### 2. What the service exposes

Describe the listeners and discovery behavior in plain language:

- Public listener: handles `ytmusic.v1.YtMusicPublic`
- Admin listener: handles `ytmusic.v1.admin.YtMusicAdmin`
- Health checks: available through the standard gRPC health service
- Reflection: available on the admin listener for `grpcurl` discovery

This section should explicitly tell users that reflection-based `grpcurl list` and `grpcurl describe` workflows should target the admin port.

### 3. Prerequisites

Document the minimum tools needed:

- Rust toolchain for local runs
- Docker or Podman for container runs
- `grpcurl` for validation and examples
- `ytmusicapi-cli` for generating `browser.json`
- Access to a Firefox session signed into YouTube Music

Keep prerequisites concrete and focused on workflows used later in the README.

### 4. Authentication setup with `ytmusicapi-cli`

This section is the centerpiece of the operator experience. It should:

- Explain what `browser.json` is at a high level: a local credential artifact used by the upstream YouTube Music client integration
- State clearly that it contains sensitive authentication material and must not be committed to version control
- Recommend Firefox for the setup flow
- Show how to install or run `ytmusicapi-cli`
- Walk through opening `https://music.youtube.com` while signed in, opening Firefox developer tools, finding a successful authenticated `POST` request to `music.youtube.com` such as a `browse` request, and copying the request headers
- Show `ytmusicapi browser` as the command that produces `browser.json`
- Explain where to store the resulting file for local runs and container mounts

The instructions should stay aligned with the upstream `ytmusicapi` browser-auth flow instead of inventing a custom process.

### 5. Configuration reference

Document each required environment variable with:

- variable name
- purpose
- example value
- whether it points to the public listener, admin listener, or credential file

The README should make it obvious that the service fails at startup if the credential path is missing, not a file, malformed, or fails the startup probe.

### 6. Local execution

Provide one complete local run example that exports all required environment variables and runs `cargo run`. The example should use loopback addresses and a local `browser.json` path so it is copy-paste friendly.

### 7. Container execution

Provide a complete container flow with:

- image build command
- `podman run` or equivalent container run example
- public and admin port mappings
- read-only bind mount for `browser.json`
- environment variable wiring inside the container

The guide should emphasize that replacing the host-side file does not activate new credentials until the admin reload RPC is called.

### 8. Practical `grpcurl` usage

Include executable examples for:

- listing services against the admin listener
- describing the public service via reflection
- checking health
- calling a representative public RPC such as `Search`
- calling `ReloadBrowserAuth` on the admin listener

Examples should use realistic local addresses and valid fully qualified service names. They should demonstrate the difference between admin-port discovery and public-port API invocation.

### 9. Credential rotation and reload workflow

Describe the operational flow:

1. Generate or replace `browser.json`
2. Ensure the service can read the updated file path
3. Call `ytmusic.v1.admin.YtMusicAdmin/ReloadBrowserAuth`
4. Confirm the response and resume traffic

Also explain that file replacement alone is insufficient because the running service keeps the previous in-memory auth context until reload succeeds.

### 10. Troubleshooting

Cover the common user-facing failure modes that can be inferred from the current implementation and tests:

- missing `browser.json`
- credential path points to a directory instead of a file
- malformed credential JSON
- startup auth probe failure
- listener bind failure because an address is already in use
- calling reflection commands on the wrong port
- replacing the credential file without invoking reload

Troubleshooting guidance should stay pragmatic and map symptoms to concrete next checks.

### 11. Further reference

End the README with links to:

- `docs/API.md`
- `proto/ytmusic/v1/public.proto`
- `proto/ytmusic/v1/admin.proto`

## `docs/API.md` Content Design

`docs/API.md` should be a compact reference index, not a field-by-field duplicate of the protobuf files.

It should include:

- overview of listener separation and package names
- fully qualified service names for `grpcurl`
- grouped summary of the public RPCs by capability area
- explanation that continuation RPCs consume tokens returned by earlier list/search operations
- admin RPC summary for credential reload
- links back to the protobuf files for exact message definitions

The public RPC summary should be grouped into:

- search and discovery
- watch playlist and playback metadata
- library listing families
- account information
- decipher

Each RPC or group should have a one-line explanation of its purpose.

## Source Constraints

The documentation update should prefer repository truths first and upstream primary documentation second.

Allowed factual sources:

- current service code and tests for runtime behavior
- protobuf files for service names and RPC inventory
- upstream `ytmusicapi` documentation for the browser-auth bootstrap flow

The README should not claim behavior that is not supported by the current implementation.

## Writing Style

- User-facing, direct, and task-oriented
- Favor runnable examples over abstract prose
- Keep sensitive-material guidance explicit but concise
- Avoid burying critical caveats such as reload requirements or admin-versus-public port differences

## Validation Criteria

The documentation change is successful when:

- a new operator can follow the README to obtain `browser.json`, start the service, and validate it with `grpcurl`
- the README clearly separates local run, container run, and reload workflows
- `docs/API.md` gives a useful API map without turning into a second protobuf dump
- examples use the actual service and package names exposed by the repository
- troubleshooting content reflects real startup and runtime failure modes present in the codebase

## Implementation Notes For The Next Planning Step

- Verify the exact `ytmusicapi-cli` installation command or invocation pattern before writing the final README examples
- Use repository paths consistently when linking docs and proto files
- Keep `README.md` detailed, but move the exhaustive RPC inventory into `docs/API.md`
