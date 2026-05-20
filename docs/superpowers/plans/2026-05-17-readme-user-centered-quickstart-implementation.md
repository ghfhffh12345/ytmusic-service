# README User-Centered Quickstart Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reframe `README.md` so it leads with a user-centered Podman quickstart using the published GHCR image, and present local execution as a minimal source-based alternative that starts at `git clone`.

**Architecture:** This is a docs-only change centered on `README.md`. The top of the README shifts from repo/build framing to user-journey framing: first successful run with `podman run`, then auth/configuration, then a concise local-from-source path, while keeping the existing `grpcurl`, reload, troubleshooting, and reference sections intact.

**Tech Stack:** Markdown, Podman runtime usage, Rust local run instructions, upstream `ytmusicapi` browser-auth flow, `rg`, `sed`, `cargo test`

---

## Planned File Structure

- `README.md`
  - primary user-facing guide; will be reordered so Podman quickstart comes first and local execution becomes a source-based alternative
- `docs/superpowers/specs/2026-05-17-readme-user-centered-quickstart-design.md`
  - approved design spec that defines the new user-centered flow

### Task 1: Rewrite the container path as a Podman-first quickstart

**Files:**
- Modify: `README.md`
- Reference: `docs/superpowers/specs/2026-05-17-readme-user-centered-quickstart-design.md`
- Test: `README.md`

- [ ] **Step 1: Confirm the current README still uses build-first container wording**

Run:

```bash
sed -n '1,95p' README.md
```

Expected: the current top of the README still contains `## Container execution`, a `podman build -t ...` block, and a separate `podman run --rm ...` block.

- [ ] **Step 2: Replace the build-first container section with a Podman quickstart section**

Update the top portion of `README.md` so it contains this structure and wording in substance:

```md
# ytmusic-service

`ytmusic-service` is a Rust gRPC wrapper around upstream `ytmusicapi` and `yt-cipher`, with separate public and admin listeners.

## Quickstart with Podman

If you already have Podman and a valid `browser.json`, this is the fastest way to run the service.

If you still need to create `browser.json`, follow [Authentication setup with ytmusicapi-cli](#authentication-setup-with-ytmusicapi-cli) first.

Podman will pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically if it is not already present locally.

```bash
podman run --rm \
  -p 50051:50051 \
  -p 50052:50052 \
  -e YTMUSIC_SERVICE_PUBLIC_ADDR=0.0.0.0:50051 \
  -e YTMUSIC_SERVICE_ADMIN_ADDR=0.0.0.0:50052 \
  -e YTMUSIC_SERVICE_BROWSER_JSON=/run/secrets/browser.json \
  -v "$PWD/secrets/browser.json:/run/secrets/browser.json:ro" \
  ghcr.io/ghfhffh12345/ytmusic-service:latest
```

Replacing the mounted file does not activate new credentials until the admin reload RPC is called.
```

This step removes the local-image build command from the default container path entirely.

- [ ] **Step 3: Keep the service overview section, but move it below the quickstart**

Ensure `README.md` still contains this section after the new quickstart:

```md
## What the service exposes

- Public gRPC API on `YTMUSIC_SERVICE_PUBLIC_ADDR` for `ytmusic.v1.YtMusicPublic`
- Separate admin gRPC API on `YTMUSIC_SERVICE_ADMIN_ADDR` for `ytmusic.v1.admin.YtMusicAdmin`
- Standard gRPC health checks on both listeners
- gRPC reflection on the admin listener for `grpcurl list` and `grpcurl describe`

Use the admin port for reflection-based discovery, and the public port for actual music RPCs.
```

- [ ] **Step 4: Verify the quickstart rewrite**

Run:

```bash
rg -n "^## (Quickstart with Podman|What the service exposes)$" README.md
rg -n "Podman will pull `ghcr.io/ghfhffh12345/ytmusic-service:latest` automatically|ghcr.io/ghfhffh12345/ytmusic-service:latest|podman build -t" README.md
```

Expected:
- `Quickstart with Podman` appears above `What the service exposes`
- the GHCR image appears in the `podman run` command
- the Podman auto-pull sentence is present
- `podman build -t` no longer appears anywhere in `README.md`

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: make README container path user-centered"
```

### Task 2: Rewrite local execution as a source-based alternative

**Files:**
- Modify: `README.md`
- Reference: `docs/superpowers/specs/2026-05-17-readme-user-centered-quickstart-design.md`
- Test: `README.md`

- [ ] **Step 1: Check the current local section wording**

Run:

```bash
rg -n "^## (Prerequisites|Local execution|Run locally from source)$" README.md
sed -n '35,90p' README.md
```

Expected: the README still uses `## Local execution` and does not yet start the source-based path with `git clone`.

- [ ] **Step 2: Rewrite the local section to start at repository clone**

Replace the local execution section with this structure and wording in substance:

```md
## Run locally from source

Use this path if you want to run the service from the repository instead of the published container image.

```bash
git clone https://github.com/ghfhffh12345/ytmusic-service.git
cd ytmusic-service

export YTMUSIC_SERVICE_PUBLIC_ADDR=127.0.0.1:50051
export YTMUSIC_SERVICE_ADMIN_ADDR=127.0.0.1:50052
export YTMUSIC_SERVICE_BROWSER_JSON="$PWD/secrets/browser.json"

cargo run
```
```

Do not add extra development-environment detail beyond this minimal path.

- [ ] **Step 3: Keep auth and configuration sections, but tune nearby wording for user flow**

Ensure these sections remain present and readable in the new order:

```md
## Authentication setup with ytmusicapi-cli
## Configuration
## Run locally from source
```

If needed, trim or adjust any nearby prose so the README reads as:
- quickstart first
- auth and configuration next
- local-from-source as an alternative path

- [ ] **Step 4: Verify the source-based local path**

Run:

```bash
rg -n "^## Run locally from source$|git clone https://github.com/ghfhffh12345/ytmusic-service.git|cd ytmusic-service|cargo run" README.md
rg -n "^## Local execution$" README.md
```

Expected:
- `Run locally from source` exists
- the exact clone URL and `cd ytmusic-service` appear
- `cargo run` remains in the local path
- the old `Local execution` heading no longer exists

- [ ] **Step 5: Commit**

```bash
git add README.md
git commit -m "docs: start local README path from repository clone"
```

### Task 3: Final documentation verification

**Files:**
- Modify: `README.md`
- Test: `README.md`
- Test: `cargo test`

- [ ] **Step 1: Run the final README structure sweep**

Run:

```bash
rg -n "^## " README.md
```

Expected: the README headings include `Quickstart with Podman`, `Authentication setup with ytmusicapi-cli`, `Configuration`, `Run locally from source`, `Practical grpcurl usage`, `Credential rotation and reload workflow`, `Troubleshooting`, and `Further reference`.

- [ ] **Step 2: Run the final content verification sweep**

Run:

```bash
rg -n "ghcr.io/ghfhffh12345/ytmusic-service:latest|Podman will pull|git clone https://github.com/ghfhffh12345/ytmusic-service.git|ytmusicapi browser|ReloadBrowserAuth" README.md
```

Expected: the README contains the published GHCR image, the Podman auto-pull note, the exact Git clone command, the auth bootstrap command, and the reload RPC reference.

- [ ] **Step 3: Run the repository tests after the docs rewrite**

Run:

```bash
cargo test
```

Expected: PASS. This is a docs-only change, so the Rust test suite should remain green.

- [ ] **Step 4: Commit any final README polish if verification exposed wording drift**

If Step 1 or Step 2 exposed a mismatch, fix `README.md` and then commit:

```bash
git add README.md
git commit -m "docs: finalize user-centered README quickstart flow"
```

If no fixes were needed, skip this step and do not create an extra commit.
