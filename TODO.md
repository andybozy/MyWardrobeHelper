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
