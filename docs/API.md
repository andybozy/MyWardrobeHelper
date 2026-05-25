# API

## Current stage

The JSON API foundation is available under `/api/v1`. It currently covers:

- health and runtime info
- item list/create/get/update
- item move and movement history
- item media list/upload
- location list/create/get
- trip list/create/get/update
- trip item list/create/update/delete
- a stable JSON error envelope

Item deletion and media deletion remain future sections.

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
- `POST /api/v1/items/:id/move`
- `GET /api/v1/items/:id/movements`

Item media:

- `GET /api/v1/items/:id/media`
- `POST /api/v1/items/:id/media`

Locations:

- `GET /api/v1/locations`
- `POST /api/v1/locations`
- `GET /api/v1/locations/:id`

Nested locations are created by passing `parent_id` when creating a location.

Trips:

- `GET /api/v1/trips`
- `POST /api/v1/trips`
- `GET /api/v1/trips/:id`
- `PATCH /api/v1/trips/:id`

Trip items:

- `GET /api/v1/trips/:id/items`
- `POST /api/v1/trips/:id/items`
- `PATCH /api/v1/trips/:id/items/:trip_item_id`
- `DELETE /api/v1/trips/:id/items/:trip_item_id`

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

`POST /api/v1/items/:id/move`

Request:

```json
{
  "to_location_id": "location-1779735257711-0",
  "reason": "packing"
}
```

Response: `200 OK`

```json
{
  "id": "movement-1779735267045-0",
  "item_id": "item-1779735257714-1",
  "from_location_id": null,
  "to_location_id": "location-1779735257711-0",
  "reason": "packing",
  "notes": null,
  "moved_at": "2026-05-25 18:54:27"
}
```

`GET /api/v1/items/:id/movements`

```json
{
  "movements": [
    {
      "id": "movement-1779735267045-0",
      "item_id": "item-1779735257714-1",
      "from_location_id": null,
      "to_location_id": "location-1779735257711-0",
      "reason": "packing",
      "notes": null,
      "moved_at": "2026-05-25 18:54:27"
    }
  ]
}
```

`PATCH /api/v1/trips/:id`

Request:

```json
{
  "luggage_type": "carry-on",
  "notes": "Two nights"
}
```

Response: `200 OK`

```json
{
  "id": "trip-1779735257716-2",
  "name": "Berlin Weekend",
  "destination": "Berlin",
  "start_date": null,
  "end_date": null,
  "trip_type": null,
  "luggage_type": "carry-on",
  "notes": "Two nights",
  "created_at": "2026-05-25 18:54:17",
  "updated_at": "2026-05-25 19:30:00"
}
```

`GET /api/v1/trips/:id/items`

```json
{
  "trip_items": [
    {
      "id": "trip-item-1779735267049-1",
      "trip_id": "trip-1779735257716-2",
      "item_id": "item-1779735257714-1",
      "item_name": "Travel Coat",
      "planned_day": "day-1",
      "status": "packed",
      "notes": "Packed in top section"
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
- `USE_MOVE_ENDPOINT`
- `INVALID_MULTIPART`
- `NO_MEDIA_FILES`
- `ITEM_NOT_FOUND`
- `LOCATION_NOT_FOUND`
- `TRIP_NOT_FOUND`
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
