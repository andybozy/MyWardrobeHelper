# GitHub Guide

This guide is the GitHub-facing overview for `MyWardrobeHelper`. It ties together the Rust backend, local browser UI, JSON API, MCP transport, and native iOS app without replacing the more detailed docs in this repository.

## What MyWardrobeHelper is

`MyWardrobeHelper` is a local-first wardrobe management system for one user.

Core rules:

- one Rust backend binary
- one SQLite database
- one local browser UI rendered by the backend
- one shared service layer used by the web UI, JSON API, MCP server, and iOS client
- mutable data stored outside the binary in the configured data directory

## Product surfaces

### Backend

The Rust backend is the source of truth for:

- data persistence
- business rules
- runtime configuration
- health checks
- backup and export workflows

Main commands:

- `cargo run --release`
- `cargo run -- help`
- `cargo run -- doctor`
- `cargo run -- backup`
- `cargo run -- export`

### Web App

The web app is server-rendered HTML served directly by the Rust backend.

Current browser areas:

- `Dashboard`
- `Items`
- `Locations`
- `Trips`
- `System Status`

The web UI supports:

- item creation and editing
- image and video upload
- hierarchical locations
- movement history
- trip planning and packing lists

### iOS App

The iOS app is a native SwiftUI client under:

- `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj`

Current iOS capabilities:

- save a backend profile and base URL
- test backend connectivity
- browse items
- open item detail
- create a basic item
- upload item images and videos
- resolve physical tags manually against the backend

The iOS app is a client only. It does not own business rules or persistence.

### JSON API

The backend exposes a local API under:

- `/api/v1`

Current areas include:

- health and server info
- items
- locations
- trips and trip items
- physical tags

See `docs/API.md` for endpoint details and `openapi/openapi.json` for the current contract.

### MCP / Codex

The backend exposes MCP for Codex-backed local tooling.

Current paths:

- unified runtime via `cargo run --release`
- standalone STDIO server via `cargo run -- mcp serve`
- local bridge via `cargo run -- mcp connect`

See `docs/MCP.md` and `docs/CODEX_SETUP.md`.

## One-command local startup

For normal local use across the backend, browser UI, iOS app, and Codex integration:

```bash
cargo run --release
```

What that does:

- auto-initializes the data directory if needed
- starts the browser UI
- starts the JSON API
- binds for LAN access by default
- starts the local MCP listener used by Codex

Default URLs:

- local browser: `http://127.0.0.1:8787`
- physical iPhone/iPad: use the printed LAN URL, typically `http://192.168.x.x:8787`

## Typical usage flow

1. Start the backend with `cargo run --release`.
2. Open the web UI and create locations first.
3. Create items and upload media.
4. Move items into locations so movement history becomes useful.
5. Create trips and packing lists.
6. Connect the iOS app to the same backend URL.
7. Use MCP/Codex against the same running backend instance.

## Repository map

- `src/` Rust backend
- `templates/` server-rendered HTML templates
- `assets/` local CSS and static assets
- `migrations/` SQLite schema migrations
- `tests/` integration and smoke tests
- `ios/` native SwiftUI app
- `docs/` detailed project docs
- `openapi/` API contract files

## Data and storage

Default data layout:

```text
.data/
  wardrobe.sqlite3
  media/
    items/
  backups/
  exports/
```

Durability commands:

- `cargo run -- backup`
- `cargo run -- export`

Important note:

- SQLite backups do not include the media file bytes
- full recovery should preserve both `wardrobe.sqlite3` and `media/items/`

## Development and verification

Backend quality gate:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Backend diagnostics:

```bash
cargo run -- doctor
```

iOS project validation on this machine can use explicit Xcode selection:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  build
```

If signing is blocked in the current shell session, compile-only validation still works with:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  build
```

## Documentation map

- `README.md` repository overview
- `docs/API.md` JSON API behavior
- `docs/MCP.md` MCP transport and tools
- `docs/CODEX_SETUP.md` Codex integration
- `docs/IOS.md` iOS app usage and validation
- `docs/TAGS.md` physical tag model and current scope
- `AGENTS.md` repository working rules

## Current boundaries

The project is intentionally opinionated:

- local-first, not cloud-first
- single-user, not multi-tenant
- server-rendered web UI, not React or Node-based
- SwiftUI iOS companion, not a second backend
- backend service layer owns logic once for every surface
