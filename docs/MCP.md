# MCP

## Why MCP exists here

MyWardrobeHelper exposes an embedded MCP server so Codex can use real local wardrobe state and actions through the same backend service layer used by the browser UI and JSON API.

This avoids browser scraping and keeps the Rust backend as the source of truth.

## Transport

Command:

- `cargo run --release`
- `cargo run -- mcp serve`
- `cargo run -- mcp connect`

Current transport:

- shared local TCP listener on `127.0.0.1:<http-port + 1>` when the default full-stack runtime is active
- STDIO for the standalone `mcp serve` command
- newline-delimited JSON-RPC messages
- no HTTP MCP transport yet

The server supports initialization, `tools/list`, `tools/call`, and `ping`.

## Recommended local workflow

For one-command local use across the browser UI, iOS, and Codex:

1. Run `cargo run --release`
2. Open the browser UI on the printed HTTP URL
3. Point the iOS app at the printed LAN URL
4. Configure Codex to run `cargo run --quiet --release -- mcp connect --data-dir .data`

`mcp connect` is a thin STDIO bridge to the local TCP listener started by the full-stack runtime, so Codex attaches to the already-running backend instead of starting a second one.

## Current tool surface

Implemented tools:

- `wardrobe.health`
- `wardrobe.list_items`
- `wardrobe.get_item`
- `wardrobe.create_item`
- `wardrobe.analyze_item_photo`
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

Current filtering support:

- `wardrobe.list_items` accepts optional `q`, `category`, `brand`, `season`, `current_location_id`, and `status` arguments
- `wardrobe.analyze_item_photo` accepts a local `image_path` argument and returns structured item-field suggestions from the Codex-backed backend

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
- photo analysis depends on a working local `codex` CLI login on the backend machine

## Related files

- MCP transport: [src/mcp/mod.rs](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/src/mcp/mod.rs)
- Shared service layer: [src/services/mod.rs](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/src/services/mod.rs)
- Codex setup: [docs/CODEX_SETUP.md](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/docs/CODEX_SETUP.md)
