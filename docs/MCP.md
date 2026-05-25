# MCP

## Why MCP exists here

MyWardrobeHelper exposes an embedded MCP server so Codex can use real local wardrobe state and actions through the same backend service layer used by the browser UI and JSON API.

This avoids browser scraping and keeps the Rust backend as the source of truth.

## Transport

Command:

- `cargo run -- mcp serve`

Current transport:

- STDIO only
- newline-delimited JSON-RPC messages
- no HTTP MCP transport yet

The server supports initialization, `tools/list`, `tools/call`, and `ping`.

## Current tool surface

Implemented tools:

- `wardrobe.health`
- `wardrobe.list_items`
- `wardrobe.get_item`
- `wardrobe.create_item`
- `wardrobe.list_locations`
- `wardrobe.create_location`
- `wardrobe.move_item`
- `wardrobe.get_item_movements`
- `wardrobe.list_trips`
- `wardrobe.create_trip`
- `wardrobe.update_trip`
- `wardrobe.get_trip`
- `wardrobe.add_trip_item`
- `wardrobe.update_trip_item`
- `wardrobe.list_trip_items`
- `wardrobe.remove_trip_item`

Required acceptance tools present for this section:

- `wardrobe.health`
- `wardrobe.list_items`
- `wardrobe.get_item`
- `wardrobe.list_locations`
- `wardrobe.move_item`
- `wardrobe.list_trips`
- `wardrobe.list_trip_items`

## Tool behavior

Design rules followed here:

- stable tool names
- shared backend service-layer calls only
- deterministic JSON outputs
- explicit JSON schemas in `tools/list`
- tool execution failures reported as MCP tool results with `isError: true`

Tool output shape:

- `content`: text block with a JSON summary
- `structuredContent`: machine-readable JSON object
- `isError: true` for tool-level failures such as invalid arguments or missing entities

## Protocol flow

Typical session:

1. Client sends `initialize`
2. Server responds with negotiated protocol version and tool capability
3. Client sends `notifications/initialized`
4. Client uses `tools/list` and `tools/call`

Supported protocol versions:

- `2025-11-25`
- `2025-06-18`
- `2025-03-26`
- `2024-11-05`

## Examples

Initialize:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"example-client","version":"1.0.0"}}}
```

List tools:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

Call a tool:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wardrobe.health","arguments":{}}}
```

## Current limitations

- no MCP resources or prompts yet
- no tool-update notifications (`listChanged` is `false`)
- no `wardrobe.update_item` yet
- no streamable HTTP transport yet

## Related files

- MCP transport: [src/mcp/mod.rs](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/src/mcp/mod.rs)
- Shared service layer: [src/services/mod.rs](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/src/services/mod.rs)
- Codex setup: [docs/CODEX_SETUP.md](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/docs/CODEX_SETUP.md)
