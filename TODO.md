# TODO

## SEC-023 - External personal ChatGPT bootstrap and parsable item import
Goal:
- Allow a user to use their own personal ChatGPT instead of the backend-side Codex quota for photo analysis, while still importing parsed item suggestions automatically into MyWardrobeHelper.

Scope:
- define one canonical import format such as `MWH_ITEM_SUGGESTION_V1` with strict JSON payload rules
- generate one reusable “copy once” bootstrap prompt that teaches ChatGPT how to answer in the required format
- add web and iOS flows to copy the bootstrap prompt, paste the returned ChatGPT result, validate it, and import it into the item form
- add backend parsing and validation so pasted output is normalized by the same shared service layer used by the Codex-backed flow
- optionally expose the canonical prompt and parser behavior through API and MCP

Acceptance criteria:
- The project has one documented canonical response format for externally generated item suggestions.
- A user can copy one bootstrap prompt and reuse it in their own ChatGPT account for repeated item-photo analysis.
- Web can paste a ChatGPT response and prefill the item form automatically.
- iOS can paste a ChatGPT response and prefill the create-item flow automatically.
- The backend validates pasted content and rejects malformed or non-conforming payloads clearly.
- The imported suggestion shape matches the same field model used by the Codex-backed analysis flow.
- `docs/API.md`, `docs/IOS.md`, `docs/CODEX_SETUP.md`, and `docs/README.md` explain the manual ChatGPT path clearly.

Notes:
- The external ChatGPT path should reduce dependency on the backend machine’s local Codex quota.
- The backend should remain the single source of truth for parsing, validation, and field normalization.
