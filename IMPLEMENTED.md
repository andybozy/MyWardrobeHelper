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

## SEC-010 - Trips and packing lists
Completed: 2026-05-25
Summary:
- Added trip create/edit/detail flows and packing-list management to the server-rendered UI.
- Added trip CRUD plus trip-item create/update/delete routes to the JSON API.
- Extended the MCP tool surface with trip update, trip-item update, and trip-item removal.
- Added tests and live smoke verification for trip persistence and packing-list status behavior across service, web, API, and MCP layers.
Acceptance criteria met:
- A user can create and edit a trip.
- A user can add/remove items to a trip.
- A trip page shows its packing list clearly.
- Packing list status is persisted.
- Equivalent service behavior is available to API and MCP.
- Tests exist for trip and trip-item persistence.
Notes:
- Packing-list updates remain intentionally simple for now: planned day, status, and notes are editable per trip item.

## SEC-011 - Native iOS app foundation and LAN server connection
Completed: 2026-05-25
Summary:
- Added a real SwiftUI iOS foundation with a stored backend server profile and a practical connection settings screen.
- Added centralized iOS networking and profile storage layers for `/api/v1/health` and `/api/v1/server-info`.
- Added local-network and local HTTP transport plist settings in the Xcode project.
- Updated iOS setup documentation and verified the Swift source with direct type-checking in this environment.
Acceptance criteria met:
- The repository contains a native iOS project.
- The app can store a configurable backend base URL.
- The app can call `/api/v1/health` and `/api/v1/server-info` and display success/failure.
- `docs/IOS.md` explains how to run the backend and connect the iOS app over LAN.
- The iOS app clearly treats the backend as the source of truth.
Notes:
- Full `xcodebuild` simulator/device builds are not available in this environment because the `iphonesimulator` SDK is not installed, so validation used `plutil` plus direct `swiftc -typecheck` instead.

## SEC-012 - iOS item browsing and editing basics
Completed: 2026-05-25
Summary:
- Added backend-backed item list and item detail screens to the native SwiftUI app.
- Added a basic create-item flow that calls `POST /api/v1/items` through the centralized `APIClient`.
- Extended the shared iOS models and networking layer so the item screens reuse the same stored server profile and API contract.
- Updated iOS documentation and verified the expanded Swift source with direct type-checking in this environment.
Acceptance criteria met:
- The app can list items from the backend.
- The app can show an item detail view.
- The app can create at least a basic item record through the API.
- Networking logic remains centralized instead of being scattered across views.
- Additional iOS unit-test targets are still not practical in this environment, but the Swift source is typechecked directly.
Notes:
- The current iOS item flow supports browsing and basic creation; richer item editing remains future work.

## SEC-013 - iOS image/video upload to MyWardrobeHelper
Completed: 2026-05-25
Summary:
- Added a centralized iOS media upload client for `/api/v1/items/:id/media`.
- Added `PhotosPicker`-based image and video selection in the item detail screen.
- Added visible per-upload progress plus backend media refresh after upload completion.
- Updated iOS documentation and validated the expanded Swift source with direct type-checking in this environment.
Acceptance criteria met:
- The app can pick one or more images from the device and upload them to an item.
- The app can pick one or more videos from the device and upload them to an item.
- Upload progress and success/failure are visible.
- The backend stores uploaded files and metadata correctly.
- The item detail flow can show the uploaded media after success.
- `docs/IOS.md` and `docs/API.md` describe the upload behavior.
Notes:
- The current iOS detail screen shows uploaded media metadata rather than full inline image/video previews.

## SEC-014 - Physical tag domain and backend future-ready contracts
Completed: 2026-05-26
Summary:
- Added explicit physical-tag domain models plus backend service and repository operations for tag registration and resolution.
- Added JSON API routes for list/create/get/resolve physical tags with the existing stable error envelope.
- Updated tag documentation and OpenAPI so the current backend contract is explicit and future-ready.
- Added tests and live smoke verification for item-bound and location-bound tag registration and resolution.
Acceptance criteria met:
- The backend has a clear `PhysicalTag` model and service direction.
- `docs/TAGS.md` explains the intended tag model and current backend status.
- There is a clean path for a tag to bind to either an item or a location.
- Added API endpoints are documented and tested.
- No fake “fully complete NFC support” claim is made.
Notes:
- The groundwork is intentionally backend-only here; iOS scanning integration remains the next dedicated section.

## SEC-015 - iOS tag-scanning groundwork and future reader integration
Completed: 2026-05-26
Summary:
- Added a `TagScannerService` abstraction to the iOS app.
- Added a native SwiftUI tag tools screen that resolves tags against the backend API.
- Added an explicit scanner placeholder path that reports live scanning as unavailable instead of faking NFC support.
- Updated iOS and tag documentation to describe the current state honestly.
Acceptance criteria met:
- The iOS app has a clear abstraction for future scanning work.
- `docs/IOS.md` and `docs/TAGS.md` describe the current status honestly.
- No fake NFC support claim is made.
- The groundwork is isolated cleanly enough to evolve without a rewrite.
Notes:
- The current iOS tag flow resolves tags manually against the backend; real NFC/QR scanning remains future work behind the same abstraction.

## SEC-016 - Search, filters, and practical dashboard views
Completed: 2026-05-26
Summary:
- Added shared item filtering support in the backend service and repository layers.
- Added item filter controls in the server-rendered item list and dashboard summary panels for category and status counts.
- Added item query-parameter filtering to the JSON API and optional filter arguments to the MCP `wardrobe.list_items` tool.
- Added tests and live verification for search/filter behavior across service, web, API, and MCP surfaces.
Acceptance criteria met:
- Users can filter items by category, brand, season, location, and status.
- The dashboard shows practical inventory summaries.
- API query parameters support common item filters.
- MCP exposes filtered listing behavior through `wardrobe.list_items`.
- Tests exist for search/filter behavior where practical.
Notes:
- Filtering is intentionally scoped to items for now; location and trip filtering can build on the same approach later.
