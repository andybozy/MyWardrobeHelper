# API

## Current stage

The JSON API is planned but not implemented yet. Initial delivery is tracked in `SEC-006`.

The backend API will be a first-class transport:

- versioned under `/api/v1`
- JSON in and JSON out
- stable, documented field names
- shared service-layer behavior with the web UI and MCP server
- suitable for the future iOS companion app

## Planned foundation

Initial endpoints:

- `GET /api/v1/health`
- `GET /api/v1/server-info`
- `GET /api/v1/items`
- `POST /api/v1/items`
- `GET /api/v1/items/:id`
- `PATCH /api/v1/items/:id`
- `GET /api/v1/locations`
- `POST /api/v1/locations`
- `GET /api/v1/locations/:id`

## Planned error contract

Errors will use a consistent JSON shape:

```json
{
  "error": {
    "code": "ITEM_NOT_FOUND",
    "message": "Item not found",
    "details": {}
  }
}
```

## Media upload direction

Item media upload is planned for `SEC-008` using `multipart/form-data`, with files stored on disk and metadata stored in SQLite.

## OpenAPI

The OpenAPI document will live in `openapi/` once the API contract is implemented.
