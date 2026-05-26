# Codex Setup

## Current stage

This repository has active lifecycle and schema initialization support. The durable workflow files are active now:

- `AGENTS.md` defines repository rules and architecture constraints.
- `TODO.md` contains unfinished top-level sections only.
- `IMPLEMENTED.md` records completed sections in chronological order.

Codex work should follow the section workflow:

1. Read `AGENTS.md`, `TODO.md`, and `IMPLEMENTED.md`.
2. Complete the highest-priority unfinished `SEC-XXX` section.
3. Update code, docs, and tracking files together.
4. Create one git commit per completed top-level section.

## Local wardrobe MCP server

The project now supports two Codex MCP paths:

- shared full-stack runtime: `cargo run --release`
- standalone STDIO runtime: `mywardrobehelper mcp serve`

Codex should use the local wardrobe MCP server for live wardrobe state and actions instead of inferring from HTML or source files.

## Project-local Codex config

A repository-scoped example is committed at `.codex/config.toml.example`.

Typical setup:

1. Copy `.codex/config.toml.example` to `.codex/config.toml`
2. Run `cargo run --release`
3. Let Codex attach with `cargo run --quiet --release -- mcp connect --data-dir .data`

If your data directory is not `.data`, edit the `args` line in `.codex/config.toml`.

The default no-argument runtime auto-initializes missing local state, binds the web/API stack for LAN access, and exposes the shared MCP listener on `127.0.0.1:<http-port + 1>`.

## Local codex binary integration

The backend now also uses the local `codex` CLI for image-based item-field suggestions.

That means the machine running the backend should satisfy:

- `codex doctor` succeeds well enough to show configured auth
- `codex login status` succeeds
- the `codex` binary is available on `PATH`

The backend shells out to `codex exec --image ... --output-schema ...` and treats the structured final message as the source of truth for field suggestions.

## Manual development workflow

- Run backend checks with `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features`.
- Start the full local stack with `cargo run --release`.
- Initialize local runtime state explicitly with `cargo run -- init` when you want migrations without starting the server.
- Validate the local runtime state with `cargo run -- doctor`.
- Inspect the local browser UI with `cargo run -- serve`.
- Smoke the standalone MCP transport with `cargo run -- mcp serve`.
- Attach a client to the shared MCP listener with `cargo run -- mcp connect`.
- Create a structured durability snapshot with `cargo run -- export`.
- Create a SQLite backup with `cargo run -- backup`.
- Inspect the current command surface with `cargo run -- help`.
- Re-run `cargo run -- init` after pulling schema changes that add new migrations.
- Review the next planned milestones in `TODO.md`.
