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
