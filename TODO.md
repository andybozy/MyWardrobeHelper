## SEC-004 - Shared domain, repository, and service layer
Objective:
- Establish the shared backend application core that all transports will use.
- Separate domain models, service logic, and persistence concerns cleanly.
- Implement initial service operations for health, item basics, location basics, and trip basics.

Acceptance criteria:
- Web/UI code does not own business logic.
- API code does not own business logic.
- MCP code does not own business logic.
- A clear backend service layer exists and is used by all later transports.
- Tests exist for core service operations where practical.

Notes:
- This section is foundational. Keep it clean.

## SEC-005 - HTTP server, base UI, navigation, and health pages
Objective:
- Implement the HTTP server and base HTML layout.
- Add simple navigation and foundational pages.
- Add a dashboard/home page and a health/status page.

Acceptance criteria:
- Running serve starts a local web server.
- The browser UI renders with a base layout and navigation.
- There is a home/dashboard page.
- There is at least one status/health page.
- CSS/JS remains minimal and local.

Notes:
- Prefer clean server-rendered templates and simple styling.

## SEC-006 - JSON API foundation and OpenAPI contract
Objective:
- Implement the versioned JSON API foundation.
- Add health endpoint and first resource endpoints.
- Introduce a documented error contract.
- Add an OpenAPI document and docs/API.md.

Acceptance criteria:
- /api/v1/health exists.
- /api/v1/server-info exists.
- The project has a documented JSON error schema.
- At least the first item and location read/write flows are available through the API.
- docs/API.md is meaningful and current.
- openapi/openapi.json or openapi/openapi.yaml exists and is updated.

Notes:
- Keep the API contract stable and boring.

## SEC-007 - Embedded MCP server and Codex project integration
Objective:
- Implement the embedded MCP server subcommand.
- Expose an initial useful tool set backed by the shared service layer.
- Add Codex-facing setup documentation and project config examples or config.
- Make the repository naturally usable by Codex through MCP.

Acceptance criteria:
- The backend supports mcp serve.
- The MCP tool layer is implemented and routed through the service layer.
- docs/MCP.md and docs/CODEX_SETUP.md clearly explain setup and usage.
- A project-local .codex/config.toml or .codex/config.toml.example exists.
- AGENTS.md includes a rule directing Codex to use the local wardrobe MCP server for live wardrobe tasks.
- The tool surface includes at least health, list items, get item, list locations, move item, list trips, and list trip items.

Notes:
- Prefer STDIO first. Keep the initial tool set high-value.

## SEC-008 - Item CRUD and backend media storage
Objective:
- Implement create, list, view, and edit flows for items.
- Implement image/video upload and storage on disk.
- Persist media metadata in SQLite.
- Show item media in the web UI and through the API where appropriate.

Acceptance criteria:
- A user can create an item with structured fields.
- A user can view and edit an item.
- A user can upload one or more images/videos for an item.
- Media files are stored on disk and metadata in SQLite.
- Tests exist for core item persistence and media metadata logic.

Notes:
- Keep media paths relative to the configured data directory.

## SEC-009 - Hierarchical locations and movement history
Objective:
- Implement nested locations.
- Allow assigning an item to a current location.
- Record movement history when an item changes location.

Acceptance criteria:
- A user can create nested locations.
- A user can assign an item to a location.
- Moving an item records a Movement entry.
- Item detail pages show current location and movement history.
- Equivalent service behavior is available to API and MCP.
- Tests exist for hierarchy and movement recording.

Notes:
- Robustness matters more than fancy visuals.

## SEC-010 - Trips and packing lists
Objective:
- Implement trip CRUD.
- Allow adding items to a trip packing list.
- Track simple packing statuses.

Acceptance criteria:
- A user can create and edit a trip.
- A user can add/remove items to a trip.
- A trip page shows its packing list clearly.
- Packing list status is persisted.
- Equivalent service behavior is available to API and MCP.
- Tests exist for trip and trip-item persistence.

Notes:
- Do not overcomplicate intelligence yet.

## SEC-011 - Native iOS app foundation and LAN server connection
Objective:
- Create the native iOS companion app structure.
- Implement server profile/base URL configuration.
- Implement connection testing against the backend API on the local network.
- Document iOS setup and local device testing.

Acceptance criteria:
- The repository contains a buildable native iOS project or workspace.
- The app can store a configurable backend base URL.
- The app can call /api/v1/health or /api/v1/server-info and display success/failure.
- docs/IOS.md explains how to run the backend and connect the iOS app over LAN.
- The iOS app clearly treats the backend as the source of truth.

Notes:
- Manual URL entry is enough initially. Bonjour discovery can wait.

## SEC-012 - iOS item browsing and editing basics
Objective:
- Implement basic iOS screens for browsing items and viewing item details.
- Add basic create/edit flows where practical using the JSON API.
- Establish reusable networking and error handling.

Acceptance criteria:
- The app can list items from the backend.
- The app can show an item detail view.
- The app can create at least a basic item record through the API.
- Networking logic is centralized, not scattered in views.
- Basic unit tests exist for API client decoding or view-model logic where practical.

Notes:
- Keep the UI clean and practical, not flashy.

## SEC-013 - iOS image/video upload to MyWardrobeHelper
Objective:
- Let the iOS app upload images and videos directly to an item in the backend.
- Implement reusable upload logic and progress handling.
- Ensure uploaded media appears back in backend-backed item views.

Acceptance criteria:
- The app can pick one or more images from the device and upload them to an item.
- The app can pick one or more videos from the device and upload them to an item.
- Upload progress and success/failure are visible.
- The backend stores uploaded files and metadata correctly.
- The item detail flow can show the uploaded media after success.
- docs/IOS.md and docs/API.md describe the upload behavior.

Notes:
- Camera capture may be added if it fits cleanly, but direct library-based upload is mandatory.

## SEC-014 - Physical tag domain and backend future-ready contracts
Objective:
- Introduce the PhysicalTag domain cleanly if not already done.
- Add backend service logic and API direction for registering and resolving physical tags.
- Document the future behavior without pretending the full feature is already finished.

Acceptance criteria:
- The backend has a clear PhysicalTag model and service direction.
- docs/TAGS.md explains the intended tag model.
- There is a clean path for a tag to bind to either an item or a location.
- If API endpoints are added, they are documented and tested.
- No fake “fully complete NFC support” claim is made unless it truly works end-to-end.

Notes:
- This section is about future-ready architecture and honest groundwork.

## SEC-015 - iOS tag-scanning groundwork and future reader integration
Objective:
- Prepare the iOS app for future physical tag reading.
- Add a clear scanner abstraction and feature boundary.
- If feasible, add a first real scanner integration for one tag type without making the architecture messy.

Acceptance criteria:
- The iOS app has a clear abstraction such as TagScannerService.
- docs/IOS.md and docs/TAGS.md describe the current status honestly.
- If a first real scanner integration is added, it is documented and tested as practical.
- If NFC is not fully implemented yet, the code and docs clearly say so.
- The groundwork does not require a rewrite later.

Notes:
- NFC is the likely main future direction. QR/barcode may be a useful intermediate step.

## SEC-016 - Search, filters, and practical dashboard views
Objective:
- Add search and useful filters for items.
- Improve the dashboard with inventory summaries and practical views.
- Expose common filtering through API query parameters.

Acceptance criteria:
- Users can filter items by at least category, brand, season, location, and status.
- The dashboard shows useful inventory summaries.
- API query parameters support common filters.
- MCP can expose filtered listing behavior where appropriate.
- Tests exist for search/filter behavior where practical.

Notes:
- Focus on usefulness, not visual complexity.

## SEC-017 - Backup, export, and data durability
Objective:
- Implement backup command(s).
- Add export capability for structured data.
- Improve durability and recovery documentation.

Acceptance criteria:
- A backup command safely creates a database backup.
- Export exists in at least JSON format.
- docs/CODEX_SETUP.md or README explains backup/export behavior.
- The repo has a clear story for data durability and recovery.

Notes:
- Keep it simple and reliable.

## SEC-018 - Quality hardening, reviewability, and repository polish
Objective:
- Ensure the project is easy for future Codex sessions and human review.
- Strengthen tests, docs, and developer ergonomics.
- Leave the repo in a clean stable state.

Acceptance criteria:
- README is current.
- AGENTS.md is current.
- docs/API.md, docs/MCP.md, docs/IOS.md, and docs/TAGS.md are current.
- fmt, clippy, and tests all pass.
- The repo is in a clean state for future section-by-section development.

Notes:
- Do not bloat the stack to satisfy this section.
