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

The repository now has its first backend lifecycle layer:

- Rust backend binary project exists and builds
- Project guidance, backlog tracking, and core docs are initialized
- CLI commands exist for `init`, `doctor`, `serve`, `backup`, `export`, and `mcp serve`
- Data directory resolution and layout management are implemented
- iOS companion app still exists as a native placeholder shell
- Database schema, HTTP server, JSON API, and MCP transport remain planned in `TODO.md`

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

- `init` creates the external data directory layout and a placeholder `wardrobe.sqlite3` file
- `doctor` checks the resolved config, filesystem layout, and writability
- `serve` resolves bind URLs and data paths, but the actual HTTP server arrives in `SEC-005`
- `backup` copies the current database file into `backups/`
- `export` writes a placeholder JSON export describing the runtime layout
- `mcp serve` reserves the embedded MCP command surface for `SEC-007`

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

`SEC-002` creates the layout and placeholder database file. SQLite schema initialization and migrations land in `SEC-003`.

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
