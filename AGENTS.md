# MyWardrobeHelper

Purpose:
- Build and maintain a local-first wardrobe management system for one user.
- Keep the Rust backend as the canonical source of truth for browser UI, JSON API, MCP, and the iOS client.

Architecture constraints:
- Single Rust backend binary on stable Rust.
- Server-rendered HTML only for the web UI; no React, Node frontend pipeline, Tailwind, or HTMX.
- SQLite for structured data and filesystem storage for media.
- Native SwiftUI iOS companion app that talks to the backend JSON API over the local network.

Working rules:
- Implement business rules once in the backend service layer; UI, API, and MCP handlers must call that layer instead of duplicating logic.
- Treat the iOS app strictly as a client; it must not become a second source of truth.
- When a task needs live wardrobe state or actions, use the local wardrobe MCP server instead of inferring from HTML or guessing from source files.
- Keep mutable data outside the binary and inside the configured data directory.

Build and test:
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-features`
- `cargo run -- help`
- `cargo run -- init`
- `cargo run -- doctor`

Tracking and commits:
- Read `TODO.md` and `IMPLEMENTED.md` before starting substantial work.
- Keep exactly one top-level `SEC-XXX` section per completion commit.
- When a section is fully complete, remove it from `TODO.md`, append it to `IMPLEMENTED.md`, and create one matching git commit.
- Do not mark a section implemented if any acceptance criteria, docs, or checks are still failing.

Documentation expectations:
- Update `README.md` whenever product scope, commands, or current status changes.
- Update `docs/API.md` for API-visible changes, `docs/MCP.md` and `docs/CODEX_SETUP.md` for MCP/Codex changes, `docs/IOS.md` for iOS-visible changes, and `docs/TAGS.md` for physical-tag changes.
