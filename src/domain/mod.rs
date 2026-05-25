#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub location_type: String,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewLocation {
    pub name: String,
    pub location_type: String,
    pub parent_id: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTrip {
    pub name: String,
    pub destination: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub trip_type: Option<String>,
    pub luggage_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub item_count: i64,
    pub location_count: i64,
    pub trip_count: i64,
}
