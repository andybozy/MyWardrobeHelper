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

The repository is in bootstrap state:

- Rust backend binary project exists and builds
- Project guidance, backlog tracking, and core docs are initialized
- iOS companion app exists as a native placeholder shell
- CLI subcommands, database schema, HTTP server, JSON API, and MCP server are planned in `TODO.md`

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

## Commands

Current bootstrap commands:

- `cargo run`
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`

Planned backend command surface:

- `mywardrobehelper serve`
- `mywardrobehelper init`
- `mywardrobehelper doctor`
- `mywardrobehelper backup`
- `mywardrobehelper export`
- `mywardrobehelper mcp serve`

## Workflow

- `TODO.md` contains unfinished top-level sections only.
- `IMPLEMENTED.md` records completed sections in chronological order.
- One completed top-level section maps to one git commit.
- Public behavior is not complete until code, docs, and verification are all updated together.

See `AGENTS.md` for repository rules and `docs/CODEX_SETUP.md` for Codex integration guidance.
