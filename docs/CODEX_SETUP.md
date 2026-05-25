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

The project will expose an embedded MCP server from the Rust backend as:

- `mywardrobehelper mcp serve`

That command is not implemented yet. It is tracked in `SEC-007`.

When `SEC-007` lands, Codex should use the local wardrobe MCP server for live wardrobe state and actions instead of inferring from HTML or source files.

## Project-local Codex config

A future-ready example is committed at `.codex/config.toml.example`. Copy it to `.codex/config.toml` once the backend exposes `mcp serve`.

Manual setup remains the current path until then.

## Manual development workflow

- Run backend checks with `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features`.
- Initialize local runtime state and apply SQLite migrations with `cargo run -- init`.
- Validate the local runtime state with `cargo run -- doctor`.
- Inspect the local browser UI with `cargo run -- serve`.
- Inspect the current command surface with `cargo run -- help`.
- Re-run `cargo run -- init` after pulling schema changes that add new migrations.
- Review the next planned milestones in `TODO.md`.
