# API

## Current stage

The JSON API foundation is now available under `/api/v1`. It currently covers:

- health and runtime info
- item list/create/get
- location list/create/get
- a stable JSON error envelope

Trip routes, item updates, location updates, movements, and media upload remain future sections.

## Versioning

- Base path: `/api/v1`
- Content type: `application/json`
- Versioning strategy: additive changes within `v1`, new base path for breaking changes later

## Implemented endpoints

Health and runtime:

- `GET /api/v1/health`
- `GET /api/v1/server-info`

Items:

- `GET /api/v1/items`
- `POST /api/v1/items`
- `GET /api/v1/items/:id`

Locations:

- `GET /api/v1/locations`
- `POST /api/v1/locations`
- `GET /api/v1/locations/:id`

## Response shape

`GET /api/v1/health`

```json
{
  "status": "ok",
  "item_count": 3,
  "location_count": 2,
  "trip_count": 1
}
```

`GET /api/v1/server-info`

```json
{
  "application": "MyWardrobeHelper",
  "version": "0.1.0",
  "bind_url": "http://127.0.0.1:8787",
  "local_url": "http://127.0.0.1:8787",
  "lan_url": null,
  "data_dir": "/path/to/.data",
  "database_file": "/path/to/.data/wardrobe.sqlite3"
}
```

`GET /api/v1/items`

```json
{
  "items": [
    {
      "id": "item-1779734473705-0",
      "name": "Weekend Coat",
      "category": "Outerwear",
      "subcategory": null,
      "brand": "Example",
      "size": null,
      "color_primary": null,
      "color_secondary": null,
      "material": null,
      "season": null,
      "formality": null,
      "status": null,
      "current_location_id": null,
      "notes": null,
      "created_at": "2026-05-25 18:41:13",
      "updated_at": "2026-05-25 18:41:13"
    }
  ]
}
```

`POST /api/v1/items`

Request:

```json
{
  "name": "Weekend Coat",
  "category": "Outerwear",
  "brand": "Example"
}
```

Response: `201 Created`

```json
{
  "id": "item-1779734473705-0",
  "name": "Weekend Coat",
  "category": "Outerwear",
  "subcategory": null,
  "brand": "Example",
  "size": null,
  "color_primary": null,
  "color_secondary": null,
  "material": null,
  "season": null,
  "formality": null,
  "status": null,
  "current_location_id": null,
  "notes": null,
  "created_at": "2026-05-25 18:41:13",
  "updated_at": "2026-05-25 18:41:13"
}
```

`POST /api/v1/locations`

Request:

```json
{
  "name": "Front Closet",
  "location_type": "Closet"
}
```

Response: `201 Created`

```json
{
  "id": "location-1779734473732-1",
  "name": "Front Closet",
  "location_type": "Closet",
  "parent_id": null,
  "notes": null,
  "created_at": "2026-05-25 18:41:13",
  "updated_at": "2026-05-25 18:41:13"
}
```

## Error contract

Every API error uses the same top-level envelope:

```json
{
  "error": {
    "code": "ITEM_NOT_FOUND",
    "message": "Item not found",
    "details": {
      "item_id": "does-not-exist"
    }
  }
}
```

Current error codes include:

- `INVALID_REQUEST`
- `ITEM_NOT_FOUND`
- `LOCATION_NOT_FOUND`
- `SERVICE_NOT_READY`
- `INTERNAL_ERROR`

## Filtering behavior

Filtering is not implemented yet. `GET /api/v1/items` and `GET /api/v1/locations` currently return the full collection. Query-parameter filtering is planned for later sections.

## Media upload behavior

Media upload is not implemented yet. Planned direction:

- endpoint under `/api/v1/items/:id/media`
- `multipart/form-data`
- file storage on disk
- metadata in SQLite

## OpenAPI

The hand-authored OpenAPI document for the current API surface lives at [openapi/openapi.json](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/openapi/openapi.json).
