use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::{
    HealthSnapshot, Item, Location, MoveItemInput, MoveItemResult, Movement, NewItem, NewLocation,
    NewTrip, NewTripItem, Trip, TripItem,
};
use crate::error::{AppError, AppResult};
use crate::repositories::SqliteWardrobeRepository;

#[derive(Debug, Clone)]
pub struct WardrobeService {
    repository: SqliteWardrobeRepository,
}

impl WardrobeService {
    pub fn new(repository: SqliteWardrobeRepository) -> Self {
        Self { repository }
    }

    pub async fn health(&self) -> AppResult<HealthSnapshot> {
        self.repository.health_snapshot().await
    }

    pub async fn create_item(&self, input: NewItem) -> AppResult<Item> {
        let normalized = NewItem {
            name: require_name("item name", input.name)?,
            category: normalize_optional(input.category),
            subcategory: normalize_optional(input.subcategory),
            brand: normalize_optional(input.brand),
            size: normalize_optional(input.size),
            color_primary: normalize_optional(input.color_primary),
            color_secondary: normalize_optional(input.color_secondary),
            material: normalize_optional(input.material),
            season: normalize_optional(input.season),
            formality: normalize_optional(input.formality),
            status: normalize_optional(input.status),
            current_location_id: normalize_optional(input.current_location_id),
            notes: normalize_optional(input.notes),
        };

        self.repository
            .create_item(&generate_id("item"), &normalized)
            .await
    }

    pub async fn list_items(&self) -> AppResult<Vec<Item>> {
        self.repository.list_items().await
    }

    pub async fn get_item(&self, id: &str) -> AppResult<Option<Item>> {
        let item_id = require_identifier("item id", id)?;
        self.repository.get_item(item_id).await
    }

    pub async fn move_item(
        &self,
        item_id: &str,
        input: MoveItemInput,
    ) -> AppResult<MoveItemResult> {
        let item_id = require_identifier("item id", item_id)?;
        let normalized = MoveItemInput {
            to_location_id: normalize_identifier_optional("location id", input.to_location_id)?,
            reason: normalize_optional(input.reason),
            notes: normalize_optional(input.notes),
        };

        self.repository
            .move_item(&generate_id("movement"), item_id, &normalized)
            .await
    }

    pub async fn get_item_movements(&self, item_id: &str) -> AppResult<Vec<Movement>> {
        let item_id = require_identifier("item id", item_id)?;
        if self.repository.get_item(item_id).await?.is_none() {
            return Err(AppError::invalid_argument(format!(
                "item `{item_id}` does not exist"
            )));
        }

        self.repository.list_item_movements(item_id).await
    }

    pub async fn create_location(&self, input: NewLocation) -> AppResult<Location> {
        let normalized = NewLocation {
            name: require_name("location name", input.name)?,
            location_type: require_name("location type", input.location_type)?,
            parent_id: normalize_optional(input.parent_id),
            notes: normalize_optional(input.notes),
        };

        self.repository
            .create_location(&generate_id("location"), &normalized)
            .await
    }

    pub async fn list_locations(&self) -> AppResult<Vec<Location>> {
        self.repository.list_locations().await
    }

    pub async fn get_location(&self, id: &str) -> AppResult<Option<Location>> {
        let location_id = require_identifier("location id", id)?;
        self.repository.get_location(location_id).await
    }

    pub async fn create_trip(&self, input: NewTrip) -> AppResult<Trip> {
        let normalized = NewTrip {
            name: require_name("trip name", input.name)?,
            destination: normalize_optional(input.destination),
            start_date: normalize_optional(input.start_date),
            end_date: normalize_optional(input.end_date),
            trip_type: normalize_optional(input.trip_type),
            luggage_type: normalize_optional(input.luggage_type),
            notes: normalize_optional(input.notes),
        };

        self.repository
            .create_trip(&generate_id("trip"), &normalized)
            .await
    }

    pub async fn list_trips(&self) -> AppResult<Vec<Trip>> {
        self.repository.list_trips().await
    }

    pub async fn get_trip(&self, id: &str) -> AppResult<Option<Trip>> {
        let trip_id = require_identifier("trip id", id)?;
        self.repository.get_trip(trip_id).await
    }

    pub async fn add_trip_item(&self, trip_id: &str, input: NewTripItem) -> AppResult<TripItem> {
        let trip_id = require_identifier("trip id", trip_id)?;
        if self.repository.get_trip(trip_id).await?.is_none() {
            return Err(AppError::invalid_argument(format!(
                "trip `{trip_id}` does not exist"
            )));
        }

        let item_id = require_identifier("item id", &input.item_id)?.to_string();
        if self.repository.get_item(&item_id).await?.is_none() {
            return Err(AppError::invalid_argument(format!(
                "item `{item_id}` does not exist"
            )));
        }

        let normalized = NewTripItem {
            item_id,
            planned_day: normalize_optional(input.planned_day),
            status: normalize_optional(input.status),
            notes: normalize_optional(input.notes),
        };

        self.repository
            .add_trip_item(&generate_id("trip-item"), trip_id, &normalized)
            .await
    }

    pub async fn list_trip_items(&self, trip_id: &str) -> AppResult<Vec<TripItem>> {
        let trip_id = require_identifier("trip id", trip_id)?;
        if self.repository.get_trip(trip_id).await?.is_none() {
            return Err(AppError::invalid_argument(format!(
                "trip `{trip_id}` does not exist"
            )));
        }

        self.repository.list_trip_items(trip_id).await
    }
}

fn require_name(field: &str, value: String) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::invalid_argument(format!(
            "{field} cannot be empty"
        )))
    } else {
        Ok(trimmed.to_string())
    }
}

fn require_identifier<'a>(field: &str, value: &'a str) -> AppResult<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(AppError::invalid_argument(format!(
            "{field} cannot be empty"
        )))
    } else {
        Ok(trimmed)
    }
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|inner| {
        let trimmed = inner.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_identifier_optional(field: &str, value: Option<String>) -> AppResult<Option<String>> {
    match value {
        Some(inner) => Ok(Some(require_identifier(field, &inner)?.to_string())),
        None => Ok(None),
    }
}

fn generate_id(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{prefix}-{millis}-{counter}")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::app::{init_app, open_context};
    use crate::config::{AppConfig, DEFAULT_HOST, DEFAULT_PORT};
    use crate::domain::{MoveItemInput, NewItem, NewLocation, NewTrip, NewTripItem};

    use super::*;

    #[tokio::test]
    async fn health_starts_with_zero_counts() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let health = service.health().await.expect("health should load");

        assert_eq!(health.item_count, 0);
        assert_eq!(health.location_count, 0);
        assert_eq!(health.trip_count, 0);
    }

    #[tokio::test]
    async fn create_and_fetch_item() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let item = service
            .create_item(NewItem {
                name: " Linen Shirt ".to_string(),
                category: Some("Tops".to_string()),
                subcategory: Some("Shirts".to_string()),
                brand: Some("Example".to_string()),
                size: Some("M".to_string()),
                color_primary: Some("White".to_string()),
                color_secondary: None,
                material: Some("Linen".to_string()),
                season: Some("Summer".to_string()),
                formality: Some("Smart casual".to_string()),
                status: Some("active".to_string()),
                current_location_id: None,
                notes: Some("Favorite".to_string()),
            })
            .await
            .expect("create item");

        assert_eq!(item.name, "Linen Shirt");

        let fetched = service
            .get_item(&item.id)
            .await
            .expect("get item")
            .expect("item should exist");

        assert_eq!(fetched.id, item.id);
        assert_eq!(service.list_items().await.expect("list items").len(), 1);
    }

    #[tokio::test]
    async fn create_and_fetch_location() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let location = service
            .create_location(NewLocation {
                name: " Bedroom Closet ".to_string(),
                location_type: "Closet".to_string(),
                parent_id: None,
                notes: Some("Primary wardrobe".to_string()),
            })
            .await
            .expect("create location");

        assert_eq!(location.name, "Bedroom Closet");
        assert_eq!(
            service
                .get_location(&location.id)
                .await
                .expect("get location")
                .expect("location should exist")
                .id,
            location.id
        );
    }

    #[tokio::test]
    async fn create_and_fetch_trip() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let trip = service
            .create_trip(NewTrip {
                name: " Venice Weekend ".to_string(),
                destination: Some("Venice".to_string()),
                start_date: Some("2026-06-01".to_string()),
                end_date: Some("2026-06-03".to_string()),
                trip_type: Some("leisure".to_string()),
                luggage_type: Some("carry-on".to_string()),
                notes: Some("Two nights".to_string()),
            })
            .await
            .expect("create trip");

        assert_eq!(trip.name, "Venice Weekend");
        assert_eq!(
            service
                .get_trip(&trip.id)
                .await
                .expect("get trip")
                .expect("trip should exist")
                .id,
            trip.id
        );
    }

    #[tokio::test]
    async fn rejects_empty_names() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let error = service
            .create_item(NewItem {
                name: "   ".to_string(),
                category: None,
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: None,
                current_location_id: None,
                notes: None,
            })
            .await
            .expect_err("empty item name should fail");

        assert!(matches!(error, AppError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn move_item_records_movement_and_updates_location() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let location = service
            .create_location(NewLocation {
                name: "Guest Room Closet".to_string(),
                location_type: "Closet".to_string(),
                parent_id: None,
                notes: None,
            })
            .await
            .expect("create location");
        let item = service
            .create_item(NewItem {
                name: "Rain Jacket".to_string(),
                category: None,
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: None,
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create item");

        let result = service
            .move_item(
                &item.id,
                MoveItemInput {
                    to_location_id: Some(location.id.clone()),
                    reason: Some("seasonal rotation".to_string()),
                    notes: None,
                },
            )
            .await
            .expect("move item");

        assert_eq!(result.item.current_location_id, Some(location.id.clone()));
        assert_eq!(result.movement.to_location_id, Some(location.id));
        assert_eq!(
            service
                .get_item_movements(&item.id)
                .await
                .expect("movements")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn add_and_list_trip_items() {
        let sandbox = ServiceSandbox::new();
        let service = sandbox.service().await;

        let item = service
            .create_item(NewItem {
                name: "Merino Tee".to_string(),
                category: None,
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: None,
                formality: None,
                status: None,
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create item");
        let trip = service
            .create_trip(NewTrip {
                name: "Turin Overnight".to_string(),
                destination: None,
                start_date: None,
                end_date: None,
                trip_type: None,
                luggage_type: None,
                notes: None,
            })
            .await
            .expect("create trip");

        let trip_item = service
            .add_trip_item(
                &trip.id,
                NewTripItem {
                    item_id: item.id.clone(),
                    planned_day: Some("day-1".to_string()),
                    status: Some("planned".to_string()),
                    notes: None,
                },
            )
            .await
            .expect("add trip item");

        let trip_items = service
            .list_trip_items(&trip.id)
            .await
            .expect("list trip items");

        assert_eq!(trip_items.len(), 1);
        assert_eq!(trip_items[0].id, trip_item.id);
        assert_eq!(trip_items[0].item_name.as_deref(), Some("Merino Tee"));
    }

    struct ServiceSandbox {
        root: PathBuf,
        data_dir: PathBuf,
    }

    impl ServiceSandbox {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "mywardrobehelper-service-test-{}-{}",
                std::process::id(),
                unique
            ));

            Self {
                data_dir: root.join("data"),
                root,
            }
        }

        async fn service(&self) -> WardrobeService {
            let config = AppConfig {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_PORT,
                data_dir: self.data_dir.clone(),
            };

            init_app(&config).await.expect("initialize database");
            open_context(config)
                .await
                .expect("open app context")
                .service
        }
    }

    impl Drop for ServiceSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
