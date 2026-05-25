# Physical Tags

## Current stage

Physical tag support is not implemented yet. The groundwork is planned in `SEC-014` for the backend and `SEC-015` for the iOS app.

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
- the backend remains the source of truth for resolution logic

## Future examples

- scan a tag on a box to open the location and its contents
- scan a tag on a garment to open the item detail
- scan a tag on luggage to open the trip-related storage location
