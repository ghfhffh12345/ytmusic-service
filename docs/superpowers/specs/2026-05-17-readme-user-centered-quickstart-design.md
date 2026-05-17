# README User-Centered Quickstart Design

## Summary

Refocus `README.md` around the user’s first successful run instead of the repository’s build perspective. The README should lead with a Podman-based quickstart that works as pasted with the published GHCR image, then present authentication setup, configuration, and a minimal local-from-source path that starts at `git clone`.

## Goals

- Make the first runnable path in the README the simplest operator path.
- Remove unnecessary build instructions from the default Podman workflow.
- Keep local execution documented, but position it as a source-based alternative.
- Preserve the existing operational content such as auth setup, `grpcurl` usage, reload, and troubleshooting.

## Non-Goals

- Do not change the service implementation, container image, or release workflow.
- Do not expand the local setup into a full development environment guide.
- Do not rewrite `docs/API.md` unless the README changes require link or wording adjustments.

## Current Problem

The README currently presents container execution in a developer-oriented way by telling the user to build the image locally before running it. That is not the shortest successful path for an operator because `podman run` can pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically.

The README also presents local execution without first guiding the user through cloning the repository, which makes the source-based path feel incomplete for a first-time user.

## Audience

### Primary audience

- Operators who want the fastest way to run the service successfully

### Secondary audience

- Users who want to run the service from source instead of using the published container image

## Deliverable

Update `README.md` so its structure and wording are user-centered, with the Podman quickstart presented before the source-based local path.

## Content Design

### 1. Intro remains short

Keep the existing short description of `ytmusic-service` and its public/admin listener split.

### 2. Podman quickstart comes first

Replace the current build-plus-run framing with a user-centered quickstart section that:

- presents the published image `ghcr.io/ghfhffh12345/ytmusic-service:latest`
- gives the `podman run --rm ...` command as the primary action
- explicitly tells the user that Podman will pull the image automatically if it is not already present locally
- keeps the required env vars and `browser.json` mount in the command

The quickstart should assume the user already has Podman installed and already has a valid `browser.json`, with links or nearby text pointing them to the auth setup section if needed.

### 3. Authentication and configuration remain in the README

Keep the `ytmusicapi` / `browser.json` instructions and the env var reference, but ensure the wording supports the new quickstart-first flow.

### 4. Local execution becomes a source-based alternative

Rename or rewrite the local section so it clearly reads as “run locally from source” rather than the default path.

The section should start with:

```bash
git clone https://github.com/ghfhffh12345/ytmusic-service.git
cd ytmusic-service
```

Then it should show:

- the required environment variable exports
- `cargo run`

Keep this path intentionally minimal.

### 5. Keep the existing operational sections

Preserve the existing sections for:

- `grpcurl` usage
- credential rotation and reload workflow
- troubleshooting
- further reference

Only adjust wording as needed so the README stays consistent with the new user-centered framing.

## Writing Style

- Lead with what the user should do next, not what the repository contains
- Prefer “run this” over “build this” unless building is actually required
- Keep the local-from-source path concise and procedural
- Avoid adding development-only detail that does not help a first-time operator

## Validation Criteria

The update is successful when:

- the first container path in the README is a copy-paste `podman run` flow using `ghcr.io/ghfhffh12345/ytmusic-service:latest`
- the README no longer tells ordinary users to run `podman build` as part of the default container path
- the source-based local path starts at `git clone https://github.com/ghfhffh12345/ytmusic-service.git`
- the README still preserves auth setup, configuration, `grpcurl`, reload, troubleshooting, and reference sections
- the overall tone reads as user-centered rather than repo-centered
