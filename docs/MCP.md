# MCP

## Why MCP exists here

MyWardrobeHelper will expose an embedded MCP server so Codex can inspect and operate on real local wardrobe data through the same backend service layer used by the UI and JSON API.

This avoids browser scraping and keeps the Rust backend as the source of truth.

## Current stage

The MCP server transport is not implemented yet. It is tracked in `SEC-007`.

Target command:

- `mywardrobehelper mcp serve`

Current placeholder behavior:

- the CLI command exists now
- it resolves the configured data directory and verifies initialization
- it does not start an MCP transport yet

Target transport:

- STDIO first

## Planned initial tool set

- `wardrobe.health`
- `wardrobe.list_items`
- `wardrobe.get_item`
- `wardrobe.create_item`
- `wardrobe.update_item`
- `wardrobe.list_locations`
- `wardrobe.create_location`
- `wardrobe.move_item`
- `wardrobe.get_item_movements`
- `wardrobe.list_trips`
- `wardrobe.create_trip`
- `wardrobe.get_trip`
- `wardrobe.add_trip_item`
- `wardrobe.update_trip_item`
- `wardrobe.list_trip_items`

## Design rules

- stable tool names
- deterministic JSON results
- explicit errors
- no duplicated business logic outside the shared service layer
