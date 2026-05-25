# MyWardrobeHelper

MyWardrobeHelper is a local-first wardrobe management system for one user. The long-term product is a single Rust backend binary that owns persistence and business rules, serves a local browser UI, exposes a stable JSON API under `/api/v1`, embeds an MCP server for Codex, and supports a native iOS companion app over the local network.

## Architecture direction

Four first-class surfaces share the same backend service layer:

- Local browser UI rendered by the Rust backend
- Versioned JSON API under `/api/v1`
- Embedded MCP server exposed by `mywardrobehelper mcp serve`
- Native SwiftUI iOS companion app that consumes the JSON API

Constraints for this repository:

- One Rust backend binary
- SQLite for structured data
- Filesystem media storage outside the binary
- Server-rendered HTML only
- No cloud-first or multi-service architecture
- No React/Node/Electron stack

## Current status

The repository now has its first backend lifecycle and browser UI layers:

- Rust backend binary project exists and builds
- Project guidance, backlog tracking, and core docs are initialized
- CLI commands exist for `init`, `doctor`, `serve`, `backup`, `export`, and `mcp serve`
- Data directory resolution and layout management are implemented
- SQLite migrations create the initial schema for items, media, locations, movements, trips, trip items, and physical tags
- Shared domain, repository, service, and app-context layers exist for health, item, location, and trip basics
- A server-rendered browser UI now ships with a dashboard and status page
- A versioned JSON API foundation now ships under `/api/v1`
- An embedded STDIO MCP server now ships for live wardrobe tool use
- Item create/view/edit flows and filesystem-backed media upload now work through the backend
- Hierarchical locations and movement history now work through the backend, web UI, API, and MCP
- Trip CRUD and packing-list status flows now work through the backend, web UI, API, and MCP
- The native iOS app now stores a backend profile and can test `/api/v1/health` and `/api/v1/server-info`
- The native iOS app can now list backend items, open item detail, and create a basic item record
- The native iOS app can now upload images and videos from the photo library into backend item media
- Physical tag registration and resolution groundwork now exists in the backend and API
- The native iOS app now has a tag tools screen plus a scanner abstraction for future NFC/QR work
- Shared item filtering and dashboard summaries now work across the backend, web UI, API, and MCP
- Backup and structured JSON export now preserve real wardrobe data for recovery workflows
- iOS companion app still exists as a native placeholder shell
- The remaining API surface and iOS client work remain planned in `TODO.md`

## Repository layout

```text
src/                  Rust backend binary and planned modules
docs/                 API, MCP, Codex, iOS, and tag documentation
ios/                  Native iOS companion app project
assets/               Static assets for the server-rendered UI
templates/            HTML templates
migrations/           SQLite migrations
openapi/              Generated or hand-authored API contract files
tests/                Backend integration and smoke tests
```

## Runtime commands

Run the backend binary with:

- `cargo run -- help`
- `cargo run -- init`
- `cargo run -- doctor`
- `cargo run -- serve`
- `cargo run -- backup`
- `cargo run -- export`
- `cargo run -- mcp serve`

Current command behavior:

- `init` creates the external data directory layout, creates `wardrobe.sqlite3`, and runs SQLite migrations
- `doctor` checks the resolved config, filesystem layout, writability, and database schema health
- `serve` starts the local HTTP server, serves the browser UI and `/api/v1`, and prints the local and LAN URLs when relevant
- `backup` copies the current SQLite database into `backups/`
- `export` writes a structured JSON snapshot of items, media metadata, locations, movements, trips, trip items, and physical tags
- `mcp serve` starts the embedded MCP server over STDIO

## Configuration

Defaults:

- host: `127.0.0.1`
- port: `8787`
- data directory: `.data`

Overrides:

- `--host HOST`
- `--port PORT`
- `--data-dir PATH`
- `--lan` to use `0.0.0.0` as the default bind host when no explicit host is provided
- `MYWARDROBEHELPER_HOST`
- `MYWARDROBEHELPER_PORT`
- `MYWARDROBEHELPER_DATA_DIR`

## Data directory layout

The backend keeps mutable state outside the binary:

```text
.data/
  wardrobe.sqlite3
  media/
    items/
  backups/
  exports/
```

SQLite migrations live in `migrations/` and are applied during `init`. The current initial schema creates:

- `locations`
- `items`
- `item_media`
- `movements`
- `trips`
- `trip_items`
- `physical_tags`

## Durability and recovery

- `cargo run -- backup` creates a timestamped copy of `wardrobe.sqlite3` under `backups/`
- `cargo run -- export` creates a structured JSON snapshot under `exports/`
- current JSON exports include media metadata but not the media file bytes themselves
- media files stay on disk under `media/items/`, so a full manual backup should include both the SQLite backup and the media tree

## Checks

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`

## Workflow

- `TODO.md` contains unfinished top-level sections only.
- `IMPLEMENTED.md` records completed sections in chronological order.
- One completed top-level section maps to one git commit.
- Public behavior is not complete until code, docs, and verification are all updated together.

See `AGENTS.md` for repository rules and `docs/CODEX_SETUP.md` for Codex integration guidance.
