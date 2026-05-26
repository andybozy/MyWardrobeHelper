# MyWardrobeHelper Documentation

This is the main documentation entry point for GitHub readers.

If you open the `docs/` folder on GitHub, this page is meant to work as the documentation landing page for:

- the Rust backend
- the local web app
- the JSON API
- the MCP/Codex integration
- Codex-powered photo analysis and item-card autocomplete
- the native iOS app
- storage, backup, export, and local development workflows

## Contents

- [What the project is](#what-the-project-is)
- [Architecture](#architecture)
- [Quick start](#quick-start)
- [Backend](#backend)
- [Web app](#web-app)
- [JSON API](#json-api)
- [MCP and Codex](#mcp-and-codex)
- [iOS app](#ios-app)
- [Data layout](#data-layout)
- [Repository structure](#repository-structure)
- [Development workflow](#development-workflow)
- [Detailed docs](#detailed-docs)
- [Current boundaries](#current-boundaries)

## What the project is

`MyWardrobeHelper` is a local-first wardrobe management system for one user.

The project is built around a single rule: the Rust backend is the source of truth.

That backend owns:

- persistence
- business rules
- browser UI rendering
- JSON API responses
- MCP tool behavior
- the state consumed by the iOS client

The iOS app is intentionally a client, not a second backend.

## Architecture

The repository has four first-class product surfaces:

1. a Rust backend binary
2. a server-rendered local browser UI
3. a versioned JSON API under `/api/v1`
4. a native SwiftUI iOS companion app

There is also a fifth integration surface for tooling:

5. an embedded MCP server for Codex and other MCP clients
6. a local Codex-bin photo analysis path used by the backend for item-field suggestions

All of these share the same backend service layer.

### Core technical constraints

- one Rust binary
- one SQLite database
- filesystem media storage
- server-rendered HTML only
- no React, Node, Electron, Tailwind, or HTMX stack
- local-first runtime, not cloud-first

## Quick start

The normal startup path is:

```bash
cargo run --release
```

This one command:

- initializes local data if needed
- starts the browser UI
- starts the JSON API
- exposes a LAN-friendly host by default
- starts the local MCP listener used by Codex
- enables the backend photo-analysis integration when the local `codex` CLI is logged in

Typical URLs:

- local browser UI: `http://127.0.0.1:8787`
- physical iPhone or iPad: use the printed LAN URL, usually `http://192.168.x.x:8787`

## Backend

The backend is implemented in Rust under `src/`.

It is responsible for:

- runtime configuration
- database initialization and migrations
- domain models
- repository access
- service-layer business logic
- media storage
- health checks
- backup and export flows

### Main backend commands

```bash
cargo run --release
cargo run -- help
cargo run -- init
cargo run -- doctor
cargo run -- serve
cargo run -- backup
cargo run -- export
cargo run -- mcp serve
cargo run -- mcp connect
```

### What each command does

- `cargo run --release`
  Starts the full local stack and auto-initializes missing data.
- `cargo run -- init`
  Creates the external data directory and runs SQLite migrations.
- `cargo run -- doctor`
  Checks filesystem readiness, schema health, runtime config, and service-layer access.
- `cargo run -- serve`
  Starts only the web server and JSON API.
- `cargo run -- backup`
  Creates a timestamped SQLite backup.
- `cargo run -- export`
  Writes a structured JSON snapshot of current wardrobe data.
- `cargo run -- mcp serve`
  Starts the standalone STDIO MCP server.
- `cargo run -- mcp connect`
  Bridges STDIO to the local MCP listener created by the unified startup path.

### Current backend domain areas

- items
- item media
- locations
- movements
- trips
- trip items
- physical tags

## Web app

The web app is rendered by the Rust backend using templates under `templates/` and assets under `assets/`.

Current browser areas:

- `Dashboard`
- `Items`
- `Locations`
- `Trips`
- `System Status`

### Current web functionality

- create items
- edit items
- analyze a photo and prefill item fields in the browser form
- upload item images and videos
- move items between locations
- inspect movement history
- create hierarchical locations
- create trips
- add and update trip packing items
- inspect backend health and runtime details

### Current navigation flow

1. Create locations first.
2. Create items.
3. Upload media on item detail pages.
4. Move items into real storage locations.
5. Create trips and attach items to packing lists.

## JSON API

The backend exposes a local API under:

- `/api/v1`

### Current endpoint groups

Health and runtime:

- `GET /api/v1/health`
- `GET /api/v1/server-info`

Items:

- `GET /api/v1/items`
- `POST /api/v1/items`
- `GET /api/v1/items/:id`
- `PATCH /api/v1/items/:id`
- `POST /api/v1/items/:id/move`
- `GET /api/v1/items/:id/movements`

Item media:

- `GET /api/v1/items/:id/media`
- `POST /api/v1/items/:id/media`

Locations:

- `GET /api/v1/locations`
- `POST /api/v1/locations`
- `GET /api/v1/locations/:id`

Trips:

- `GET /api/v1/trips`
- `POST /api/v1/trips`
- `GET /api/v1/trips/:id`
- `PATCH /api/v1/trips/:id`

Trip items:

- `GET /api/v1/trips/:id/items`
- `POST /api/v1/trips/:id/items`
- `PATCH /api/v1/trips/:id/items/:trip_item_id`
- `DELETE /api/v1/trips/:id/items/:trip_item_id`

Tags:

- `GET /api/v1/tags`
- `POST /api/v1/tags`
- `GET /api/v1/tags/:id`
- `POST /api/v1/tags/resolve`

For endpoint details, examples, and response shapes, see [API.md](API.md).

## MCP and Codex

The backend also exposes an MCP tool surface for Codex and compatible clients.

### Current MCP modes

- unified startup path via `cargo run --release`
- standalone STDIO MCP server via `cargo run -- mcp serve`
- local bridge via `cargo run -- mcp connect`

### Current MCP tool groups

- health
- item listing and item fetch
- item creation
- item photo analysis from a local image path
- location listing and creation
- item movement and movement history
- trip listing and trip fetch
- trip creation and trip update
- trip item add, update, list, and remove

### Why MCP exists here

It allows Codex to read and act on real local wardrobe state through the same service layer used by the web UI and API.

That avoids:

- HTML scraping
- duplicated business logic
- backend state drifting away from tool behavior

For MCP transport details and examples, see [MCP.md](MCP.md).
For Codex setup, see [CODEX_SETUP.md](CODEX_SETUP.md).

## iOS app

The iOS client lives at:

- `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj`

It is a native SwiftUI application that talks to the backend API over the local network.

### Current iOS functionality

- store a backend profile
- store a backend base URL
- test connectivity
- show backend health
- show backend runtime info
- list items
- open item detail
- create a basic item
- analyze a selected photo during item creation and prefill fields
- upload images and videos
- resolve physical tags manually

### iOS setup

1. Start the backend:

```bash
cargo run --release
```

2. Open the Xcode project:

- `ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj`

3. Run the app on a simulator or a physical device.

4. In the app, set the backend URL:

- simulator on same Mac: `http://127.0.0.1:8787`
- physical iPhone on same LAN: use the printed LAN URL

### iOS validation

If full Xcode is installed but not the active developer directory:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  build
```

If code signing is blocked in the current shell session:

```bash
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild \
  -project ios/MyWardrobeHelperiOS/MyWardrobeHelperiOS/MyWardrobeHelperiOS.xcodeproj \
  -scheme MyWardrobeHelperiOS \
  -destination 'id=<device-id>' \
  CODE_SIGNING_ALLOWED=NO \
  CODE_SIGNING_REQUIRED=NO \
  build
```

For more iOS-specific detail, see [IOS.md](IOS.md).

## Data layout

Default runtime storage:

```text
.data/
  wardrobe.sqlite3
  media/
    items/
  backups/
  exports/
```

### What lives where

- `wardrobe.sqlite3`
  Structured wardrobe data
- `media/items/`
  Uploaded item images and videos
- `backups/`
  Timestamped SQLite copies
- `exports/`
  Structured JSON exports

### Backup and recovery notes

- `cargo run -- backup` does not include media file bytes
- `cargo run -- export` includes structured metadata, not media binaries
- a full manual recovery should preserve both the database and the media tree

## Repository structure

```text
src/                  Rust backend
templates/            Server-rendered HTML templates
assets/               Static frontend assets
migrations/           SQLite schema migrations
tests/                Integration and smoke tests
ios/                  Native SwiftUI app
docs/                 Documentation
openapi/              API contract files
```

## Development workflow

### Backend quality gate

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

### Runtime validation

```bash
cargo run -- doctor
```

### Practical local workflow

1. Start the full system with `cargo run --release`.
2. Open the browser UI.
3. Connect the iOS app to the same backend instance.
4. Use Codex through the MCP bridge against the same backend instance.
5. Use `backup` and `export` for durability.

## Detailed docs

- [API.md](API.md)
  JSON API endpoints, examples, and error shapes
- [MCP.md](MCP.md)
  MCP transport and tool surface
- [CODEX_SETUP.md](CODEX_SETUP.md)
  Codex integration and local MCP setup
- [IOS.md](IOS.md)
  iOS app behavior and build validation
- [TAGS.md](TAGS.md)
  Physical tag model and current scope

## Current boundaries

The project is intentionally constrained.

- local-first, not cloud-first
- single-user, not multi-tenant
- server-rendered web UI, not React or Node based
- iOS app is a client, not a second source of truth
- business rules belong in the backend service layer once

Current notable limitations:

- richer iOS item editing is still limited
- live NFC and QR scanning is not implemented yet
- some signed device-build workflows still depend on local Xcode signing state
