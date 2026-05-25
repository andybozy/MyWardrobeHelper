# API

## Current stage

The JSON API foundation is available under `/api/v1`. It currently covers:

- health and runtime info
- item list/create/get/update
- item media list/upload
- location list/create/get
- a stable JSON error envelope

Trip routes, movement routes, item deletion, and media deletion remain future sections.

## Versioning

- Base path: `/api/v1`
- Content type: `application/json`
- Multipart uploads: `multipart/form-data`
- Versioning strategy: additive changes within `v1`, new base path for breaking changes later

## Implemented endpoints

Health and runtime:

- `GET /api/v1/health`
- `GET /api/v1/server-info`

Items:

- `GET /api/v1/items`
- `POST /api/v1/items`
- `GET /api/v1/items/:id`
- `PATCH /api/v1/items/:id`

Item media:

- `GET /api/v1/items/:id/media`
- `POST /api/v1/items/:id/media`

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

`PATCH /api/v1/items/:id`

Request:

```json
{
  "brand": "Example",
  "status": "ready"
}
```

Response: `200 OK`

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
  "status": "ready",
  "current_location_id": null,
  "notes": null,
  "created_at": "2026-05-25 18:41:13",
  "updated_at": "2026-05-25 18:59:01"
}
```

`GET /api/v1/items/:id/media`

```json
{
  "media": [
    {
      "id": "media-1779736001000-0",
      "item_id": "item-1779734473705-0",
      "media_kind": "image",
      "relative_file_path": "media/items/item-1779734473705-0/media-1779736001000-0.jpg",
      "original_filename": "coat.jpg",
      "mime_type": "image/jpeg",
      "file_size_bytes": 16,
      "duration_ms": null,
      "width": null,
      "height": null,
      "caption": "Front view",
      "sort_order": 0,
      "created_at": "2026-05-25 18:59:01"
    }
  ]
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
- `INVALID_MULTIPART`
- `NO_MEDIA_FILES`
- `ITEM_NOT_FOUND`
- `LOCATION_NOT_FOUND`
- `SERVICE_NOT_READY`
- `INTERNAL_ERROR`

## Filtering behavior

Filtering is not implemented yet. `GET /api/v1/items` and `GET /api/v1/locations` currently return the full collection. Query-parameter filtering is planned for later sections.

## Media upload behavior

`POST /api/v1/items/:id/media` accepts `multipart/form-data` with:

- one or more `file` parts
- optional `caption` text part

Backend behavior:

- validates that the item exists
- accepts `image/*` and `video/*`
- stores files under `media/items/<item-id>/`
- persists metadata in SQLite
- returns the created media records

Current limitations:

- width, height, and duration metadata are not extracted yet
- no delete endpoint yet
- no thumbnail generation yet

## OpenAPI

The hand-authored OpenAPI document for the current API surface lives at [openapi/openapi.json](/Users/andreabozzato/PycharmProjects/MyWardrobeHelper/openapi/openapi.json).
