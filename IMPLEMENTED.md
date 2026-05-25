# Implemented Sections

Completed top-level sections are recorded here in chronological order once their acceptance criteria, docs, and verification are all complete.

## SEC-001 - Repository bootstrap and durable project guidance
Completed: 2026-05-25
Summary:
- Initialized the Rust single-binary repository scaffold and placeholder backend module layout.
- Added durable repository guidance, backlog tracking, bootstrap docs, and a native iOS placeholder shell description.
- Cleaned repository hygiene for local data and generated Xcode user files.
Acceptance criteria met:
- The repository builds as a Rust backend binary project.
- AGENTS.md exists and reflects the stable repo rules.
- TODO.md and IMPLEMENTED.md exist in the required format.
- README.md exists and documents the current architecture direction.
- docs/CODEX_SETUP.md, docs/API.md, docs/MCP.md, docs/IOS.md, and docs/TAGS.md exist.
- The repository clearly contains both backend and iOS project directions.
- No Node/Django/React/Tailwind/HTMX stack was introduced.
Notes:
- CLI subcommands, database setup, API, MCP, and richer iOS functionality remain in TODO.md for later sections.

## SEC-002 - CLI, config, data directory, and backend application lifecycle
Completed: 2026-05-25
Summary:
- Replaced the bootstrap stub with a real CLI parser and clean user-facing error handling.
- Added runtime config resolution for host, port, and external data directory paths.
- Implemented `init`, `doctor`, and placeholder `serve`, `backup`, `export`, and `mcp serve` flows backed by a deterministic filesystem layout.
- Documented runtime defaults, command behavior, and data layout in the repository docs.
Acceptance criteria met:
- The backend supports placeholder versions of `init`, `serve`, `doctor`, `backup`, `export`, and `mcp serve`.
- The app resolves and creates the configured data directory layout.
- `README.md` documents the runtime commands and data layout.
- Errors are handled cleanly without panicking for normal user mistakes.
Notes:
- The database file created in this section is a placeholder path anchor; SQLite schema creation and migrations are deferred to `SEC-003`.
