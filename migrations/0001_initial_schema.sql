CREATE TABLE locations (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    location_type TEXT NOT NULL,
    parent_id TEXT REFERENCES locations(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE items (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    subcategory TEXT,
    brand TEXT,
    size TEXT,
    color_primary TEXT,
    color_secondary TEXT,
    material TEXT,
    season TEXT,
    formality TEXT,
    status TEXT,
    current_location_id TEXT REFERENCES locations(id) ON DELETE SET NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE item_media (
    id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    media_kind TEXT NOT NULL,
    relative_file_path TEXT NOT NULL,
    original_filename TEXT,
    mime_type TEXT,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,
    duration_ms INTEGER,
    width INTEGER,
    height INTEGER,
    caption TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE movements (
    id TEXT PRIMARY KEY NOT NULL,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    from_location_id TEXT REFERENCES locations(id) ON DELETE SET NULL,
    to_location_id TEXT REFERENCES locations(id) ON DELETE SET NULL,
    reason TEXT,
    notes TEXT,
    moved_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE trips (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    destination TEXT,
    start_date TEXT,
    end_date TEXT,
    trip_type TEXT,
    luggage_type TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE trip_items (
    id TEXT PRIMARY KEY NOT NULL,
    trip_id TEXT NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    planned_day TEXT,
    status TEXT,
    notes TEXT
);

CREATE TABLE physical_tags (
    id TEXT PRIMARY KEY NOT NULL,
    tag_type TEXT NOT NULL,
    external_identifier TEXT NOT NULL,
    label TEXT,
    bound_entity_type TEXT,
    bound_entity_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tag_type, external_identifier)
);

CREATE INDEX idx_locations_parent_id ON locations(parent_id);
CREATE INDEX idx_items_current_location_id ON items(current_location_id);
CREATE INDEX idx_item_media_item_id ON item_media(item_id);
CREATE INDEX idx_movements_item_id ON movements(item_id);
CREATE INDEX idx_movements_to_location_id ON movements(to_location_id);
CREATE INDEX idx_trip_items_trip_id ON trip_items(trip_id);
CREATE INDEX idx_trip_items_item_id ON trip_items(item_id);
