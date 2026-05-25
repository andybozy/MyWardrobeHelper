# Physical Tags

## Current stage

Backend groundwork is now implemented:

- physical tags are stored in SQLite
- a tag can bind to either an item or a location
- the backend can register a tag and resolve it by `(tag_type, external_identifier)`
- the JSON API exposes list/create/get/resolve tag routes

The iOS groundwork is now in place in `SEC-015`:

- the app has a `TagScannerService` abstraction
- a tag tools screen can resolve tags against the backend
- live scanner implementations are still intentionally absent

## Product direction

Physical tags will allow quick resolution of either:

- an item
- a hierarchical storage location

Planned tag types:

- `nfc`
- `qr`
- `barcode`
- `other`

## Binding rules

- a tag binds to exactly one entity
- the bound entity can be either an item or a location
- the backend validates that the bound entity exists at registration time
- the backend remains the source of truth for resolution logic

Current backend fields:

- `tag_type`
- `external_identifier`
- `label`
- `bound_entity_type`
- `bound_entity_id`
- `notes`

## Current API surface

- `GET /api/v1/tags`
- `POST /api/v1/tags`
- `GET /api/v1/tags/:id`
- `POST /api/v1/tags/resolve`

Example resolve request:

```json
{
  "tag_type": "nfc",
  "external_identifier": "04-A2-88-FF"
}
```

Example resolve response:

```json
{
  "tag": {
    "id": "tag-1779739000000-0",
    "tag_type": "nfc",
    "external_identifier": "04-A2-88-FF",
    "label": "Overshirt NFC",
    "bound_entity_type": "item",
    "bound_entity_id": "item-1779738000000-0",
    "notes": null,
    "created_at": "2026-05-26 10:00:00",
    "updated_at": "2026-05-26 10:00:00"
  },
  "entity_name": "Corduroy Overshirt"
}
```

## Honest limitations

- no live iOS scanner integration yet
- no NFC write flow
- no QR/barcode camera scanner yet
- no tag editing or deletion API yet
- no MCP tools for tags yet

## Future examples

- scan a tag on a box to open the location and its contents
- scan a tag on a garment to open the item detail
- scan a tag on luggage to open the trip-related storage location
