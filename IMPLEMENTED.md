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

## SEC-003 - Database schema, migrations, and startup initialization
Completed: 2026-05-25
Summary:
- Added SQLite support with a migration-backed initial schema.
- Updated `init` to create a real SQLite database and apply migrations.
- Extended runtime validation so the lifecycle commands verify schema readiness instead of only filesystem layout.
- Added integration coverage for schema creation and clear `serve` guidance on uninitialized data directories.
Acceptance criteria met:
- Migrations exist and can create the initial schema.
- Running `init` creates a usable SQLite database.
- Running `serve` against an empty data directory guides the user clearly.
- Basic integration tests exist for schema initialization.
Notes:
- The first schema defines `locations`, `items`, `item_media`, `movements`, `trips`, `trip_items`, and `physical_tags`.

## SEC-004 - Shared domain, repository, and service layer
Completed: 2026-05-25
Summary:
- Added explicit domain models for items, locations, trips, and service-level health snapshots.
- Added a concrete SQLite repository layer for create/list/get basics across items, locations, and trips.
- Added a shared `WardrobeService` and `AppContext` so later transports can reuse the same backend application core.
- Wired the runtime `doctor` flow through the shared service layer and added tests for the core service operations.
Acceptance criteria met:
- Web/UI code does not own business logic.
- API code does not own business logic.
- MCP code does not own business logic.
- A clear backend service layer exists and is positioned for reuse by later transports.
- Tests exist for core service operations where practical.
Notes:
- Current service operations cover health, create/list/get item, create/list/get location, and create/list/get trip.

## SEC-005 - HTTP server, base UI, navigation, and health pages
Completed: 2026-05-25
Summary:
- Added an `axum`-based local HTTP server and server-rendered UI using `askama` templates.
- Added a dashboard page backed by the shared service layer plus a runtime status page backed by doctor checks.
- Added a local stylesheet asset and kept the browser UI free of frontend build tooling.
- Added router-level tests and live `serve` smoke verification for the dashboard and status pages.
Acceptance criteria met:
- Running `serve` starts a local web server.
- The browser UI renders with a base layout and navigation.
- There is a home/dashboard page.
- There is a status/health page.
- CSS/JS remains minimal and local.
Notes:
- The web transport now exists; the JSON API and embedded MCP transports remain the next planned backend surfaces.

## SEC-006 - JSON API foundation and OpenAPI contract
Completed: 2026-05-25
Summary:
- Added a versioned `/api/v1` router with health and server-info endpoints plus item and location list/create/get flows.
- Added a stable JSON error envelope with explicit API error codes.
- Nested the API under the local HTTP server so the browser UI and API share the same backend runtime.
- Added API endpoint tests, current API docs, and a hand-authored OpenAPI document.
Acceptance criteria met:
- `/api/v1/health` exists.
- `/api/v1/server-info` exists.
- The project has a documented JSON error schema.
- First item and location read/write flows are available through the API.
- `docs/API.md` is meaningful and current.
- `openapi/openapi.json` exists and reflects the current API surface.
Notes:
- The initial API focuses on health plus item/location basics; trip, movement, media, and update routes remain future sections.

## SEC-007 - Embedded MCP server and Codex project integration
Completed: 2026-05-25
Summary:
- Added a real embedded MCP server on `mcp serve` using STDIO and newline-delimited JSON-RPC.
- Implemented MCP initialization, `tools/list`, `tools/call`, and a high-value wardrobe tool surface backed by the shared service layer.
- Extended the shared backend core with movement and trip-item operations needed by the MCP tools.
- Updated MCP and Codex setup docs plus the project-local config example for trusted checkouts.
Acceptance criteria met:
- The backend supports `mcp serve`.
- The MCP tool layer is implemented and routed through the service layer.
- `docs/MCP.md` and `docs/CODEX_SETUP.md` clearly explain setup and usage.
- A project-local `.codex/config.toml.example` exists and is current.
- `AGENTS.md` includes the rule directing Codex to use the local wardrobe MCP server for live wardrobe tasks.
- The tool surface includes health, list items, get item, list locations, move item, list trips, and list trip items.
Notes:
- The initial MCP server is STDIO-only and focuses on tools; prompts and resources remain out of scope for now.

## SEC-008 - Item CRUD and backend media storage
Completed: 2026-05-25
Summary:
- Added item list/detail/create/edit flows to the server-rendered UI and item update/media routes to the JSON API.
- Added filesystem-backed media storage plus SQLite metadata persistence for item images and videos.
- Added static media serving for the local browser UI and multipart upload handling for both the UI and API.
- Added service, API, and web tests covering item update, media persistence, upload, and retrieval behavior.
Acceptance criteria met:
- A user can create an item with structured fields.
- A user can view and edit an item.
- A user can upload one or more images/videos for an item.
- Media files are stored on disk and metadata in SQLite.
- Tests exist for core item persistence and media metadata logic.
Notes:
- Media paths remain relative to the configured data directory under `media/items/<item-id>/`.

## SEC-009 - Hierarchical locations and movement history
Completed: 2026-05-25
Summary:
- Added a dedicated locations page with nested parent selection and hierarchy rendering in the server-rendered UI.
- Added dedicated item move flows in the web UI and JSON API so location changes always record movement history.
- Added current-location and movement-history rendering to item detail pages using human-readable location paths.
- Added tests and live smoke verification for nested locations, movement recording, and item movement history through the web and API surfaces.
Acceptance criteria met:
- A user can create nested locations.
- A user can assign an item to a location.
- Moving an item records a `Movement` entry.
- Item detail pages show current location and movement history.
- Equivalent service behavior is available to API and MCP.
- Tests exist for hierarchy and movement recording.
Notes:
- Direct location changes are intentionally routed through the dedicated move endpoints to keep movement history trustworthy.
