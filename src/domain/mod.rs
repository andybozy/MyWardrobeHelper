use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub brand: Option<String>,
    pub size: Option<String>,
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub material: Option<String>,
    pub season: Option<String>,
    pub formality: Option<String>,
    pub status: Option<String>,
    pub current_location_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewItem {
    pub name: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub brand: Option<String>,
    pub size: Option<String>,
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub material: Option<String>,
    pub season: Option<String>,
    pub formality: Option<String>,
    pub status: Option<String>,
    pub current_location_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateItemInput {
    pub name: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub brand: Option<String>,
    pub size: Option<String>,
    pub color_primary: Option<String>,
    pub color_secondary: Option<String>,
    pub material: Option<String>,
    pub season: Option<String>,
    pub formality: Option<String>,
    pub status: Option<String>,
    pub current_location_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemMedia {
    pub id: String,
    pub item_id: String,
    pub media_kind: String,
    pub relative_file_path: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size_bytes: i64,
    pub duration_ms: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub caption: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewItemMediaInput {
    pub media_kind: String,
    pub original_filename: Option<String>,
    pub mime_type: String,
    pub caption: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub location_type: String,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLocation {
    pub name: String,
    pub location_type: String,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trip {
    pub id: String,
    pub name: String,
    pub destination: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub trip_type: Option<String>,
    pub luggage_type: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTrip {
    pub name: String,
    pub destination: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub trip_type: Option<String>,
    pub luggage_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Movement {
    pub id: String,
    pub item_id: String,
    pub from_location_id: Option<String>,
    pub to_location_id: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
    pub moved_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveItemInput {
    pub to_location_id: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveItemResult {
    pub item: Item,
    pub movement: Movement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripItem {
    pub id: String,
    pub trip_id: String,
    pub item_id: String,
    pub item_name: Option<String>,
    pub planned_day: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewTripItem {
    pub item_id: String,
    pub planned_day: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub item_count: i64,
    pub location_count: i64,
    pub trip_count: i64,
}
