use std::path::PathBuf;

use sqlx::{Connection, Row, SqliteConnection};

use crate::db;
use crate::domain::{
    HealthSnapshot, Item, Location, MoveItemInput, MoveItemResult, Movement, NewItem, NewLocation,
    NewTrip, NewTripItem, Trip, TripItem,
};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct SqliteWardrobeRepository {
    database_file: PathBuf,
}

impl SqliteWardrobeRepository {
    pub fn new(database_file: PathBuf) -> Self {
        Self { database_file }
    }

    pub async fn create_item(&self, id: &str, input: &NewItem) -> AppResult<Item> {
        let mut connection = self.connect().await?;
        sqlx::query(
            "INSERT INTO items (
                id, name, category, subcategory, brand, size, color_primary, color_secondary,
                material, season, formality, status, current_location_id, notes
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.category)
        .bind(&input.subcategory)
        .bind(&input.brand)
        .bind(&input.size)
        .bind(&input.color_primary)
        .bind(&input.color_secondary)
        .bind(&input.material)
        .bind(&input.season)
        .bind(&input.formality)
        .bind(&input.status)
        .bind(&input.current_location_id)
        .bind(&input.notes)
        .execute(&mut connection)
        .await
        .map_err(|error| AppError::database("insert item", error))?;

        self.get_item(id)
            .await?
            .ok_or_else(|| AppError::database("load item after insert", sqlx::Error::RowNotFound))
    }

    pub async fn list_items(&self) -> AppResult<Vec<Item>> {
        let mut connection = self.connect().await?;
        let rows = sqlx::query(
            "SELECT
                id, name, category, subcategory, brand, size, color_primary, color_secondary,
                material, season, formality, status, current_location_id, notes, created_at, updated_at
             FROM items
             ORDER BY created_at, id",
        )
        .fetch_all(&mut connection)
        .await
        .map_err(|error| AppError::database("list items", error))?;

        rows.into_iter()
            .map(map_item_row)
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn get_item(&self, id: &str) -> AppResult<Option<Item>> {
        let mut connection = self.connect().await?;
        let row = sqlx::query(
            "SELECT
                id, name, category, subcategory, brand, size, color_primary, color_secondary,
                material, season, formality, status, current_location_id, notes, created_at, updated_at
             FROM items
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut connection)
        .await
        .map_err(|error| AppError::database("get item", error))?;

        row.map(map_item_row).transpose()
    }

    pub async fn move_item(
        &self,
        movement_id: &str,
        item_id: &str,
        input: &MoveItemInput,
    ) -> AppResult<MoveItemResult> {
        let mut connection = self.connect().await?;
        let mut transaction = connection
            .begin()
            .await
            .map_err(|error| AppError::database("start item move transaction", error))?;

        let Some(current_item_row) = sqlx::query(
            "SELECT
                id, name, category, subcategory, brand, size, color_primary, color_secondary,
                material, season, formality, status, current_location_id, notes, created_at, updated_at
             FROM items
             WHERE id = ?",
        )
        .bind(item_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| AppError::database("load item before move", error))?
        else {
            return Err(AppError::invalid_argument(format!(
                "item `{item_id}` does not exist"
            )));
        };
        let current_item = map_item_row(current_item_row)?;

        if let Some(location_id) = &input.to_location_id {
            let location_exists = sqlx::query("SELECT 1 FROM locations WHERE id = ?")
                .bind(location_id)
                .fetch_optional(&mut *transaction)
                .await
                .map_err(|error| AppError::database("validate destination location", error))?
                .is_some();

            if !location_exists {
                return Err(AppError::invalid_argument(format!(
                    "location `{location_id}` does not exist"
                )));
            }
        }

        sqlx::query(
            "UPDATE items
             SET current_location_id = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(&input.to_location_id)
        .bind(item_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::database("update item location", error))?;

        sqlx::query(
            "INSERT INTO movements (id, item_id, from_location_id, to_location_id, reason, notes)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(movement_id)
        .bind(item_id)
        .bind(&current_item.current_location_id)
        .bind(&input.to_location_id)
        .bind(&input.reason)
        .bind(&input.notes)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::database("insert movement", error))?;

        let updated_item_row = sqlx::query(
            "SELECT
                id, name, category, subcategory, brand, size, color_primary, color_secondary,
                material, season, formality, status, current_location_id, notes, created_at, updated_at
             FROM items
             WHERE id = ?",
        )
        .bind(item_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| AppError::database("load item after move", error))?;
        let movement_row = sqlx::query(
            "SELECT id, item_id, from_location_id, to_location_id, reason, notes, moved_at
             FROM movements
             WHERE id = ?",
        )
        .bind(movement_id)
        .fetch_one(&mut *transaction)
        .await
        .map_err(|error| AppError::database("load movement after insert", error))?;

        transaction
            .commit()
            .await
            .map_err(|error| AppError::database("commit item move transaction", error))?;

        Ok(MoveItemResult {
            item: map_item_row(updated_item_row)?,
            movement: map_movement_row(movement_row)?,
        })
    }

    pub async fn list_item_movements(&self, item_id: &str) -> AppResult<Vec<Movement>> {
        let mut connection = self.connect().await?;
        let rows = sqlx::query(
            "SELECT id, item_id, from_location_id, to_location_id, reason, notes, moved_at
             FROM movements
             WHERE item_id = ?
             ORDER BY moved_at DESC, id DESC",
        )
        .bind(item_id)
        .fetch_all(&mut connection)
        .await
        .map_err(|error| AppError::database("list item movements", error))?;

        rows.into_iter()
            .map(map_movement_row)
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn create_location(&self, id: &str, input: &NewLocation) -> AppResult<Location> {
        let mut connection = self.connect().await?;
        sqlx::query(
            "INSERT INTO locations (id, name, location_type, parent_id, notes)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.location_type)
        .bind(&input.parent_id)
        .bind(&input.notes)
        .execute(&mut connection)
        .await
        .map_err(|error| AppError::database("insert location", error))?;

        self.get_location(id).await?.ok_or_else(|| {
            AppError::database("load location after insert", sqlx::Error::RowNotFound)
        })
    }

    pub async fn list_locations(&self) -> AppResult<Vec<Location>> {
        let mut connection = self.connect().await?;
        let rows = sqlx::query(
            "SELECT id, name, location_type, parent_id, notes, created_at, updated_at
             FROM locations
             ORDER BY created_at, id",
        )
        .fetch_all(&mut connection)
        .await
        .map_err(|error| AppError::database("list locations", error))?;

        rows.into_iter()
            .map(map_location_row)
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn get_location(&self, id: &str) -> AppResult<Option<Location>> {
        let mut connection = self.connect().await?;
        let row = sqlx::query(
            "SELECT id, name, location_type, parent_id, notes, created_at, updated_at
             FROM locations
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut connection)
        .await
        .map_err(|error| AppError::database("get location", error))?;

        row.map(map_location_row).transpose()
    }

    pub async fn create_trip(&self, id: &str, input: &NewTrip) -> AppResult<Trip> {
        let mut connection = self.connect().await?;
        sqlx::query(
            "INSERT INTO trips (
                id, name, destination, start_date, end_date, trip_type, luggage_type, notes
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.destination)
        .bind(&input.start_date)
        .bind(&input.end_date)
        .bind(&input.trip_type)
        .bind(&input.luggage_type)
        .bind(&input.notes)
        .execute(&mut connection)
        .await
        .map_err(|error| AppError::database("insert trip", error))?;

        self.get_trip(id)
            .await?
            .ok_or_else(|| AppError::database("load trip after insert", sqlx::Error::RowNotFound))
    }

    pub async fn list_trips(&self) -> AppResult<Vec<Trip>> {
        let mut connection = self.connect().await?;
        let rows = sqlx::query(
            "SELECT
                id, name, destination, start_date, end_date, trip_type, luggage_type, notes,
                created_at, updated_at
             FROM trips
             ORDER BY created_at, id",
        )
        .fetch_all(&mut connection)
        .await
        .map_err(|error| AppError::database("list trips", error))?;

        rows.into_iter()
            .map(map_trip_row)
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn get_trip(&self, id: &str) -> AppResult<Option<Trip>> {
        let mut connection = self.connect().await?;
        let row = sqlx::query(
            "SELECT
                id, name, destination, start_date, end_date, trip_type, luggage_type, notes,
                created_at, updated_at
             FROM trips
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&mut connection)
        .await
        .map_err(|error| AppError::database("get trip", error))?;

        row.map(map_trip_row).transpose()
    }

    pub async fn add_trip_item(
        &self,
        trip_item_id: &str,
        trip_id: &str,
        input: &NewTripItem,
    ) -> AppResult<TripItem> {
        let mut connection = self.connect().await?;
        sqlx::query(
            "INSERT INTO trip_items (id, trip_id, item_id, planned_day, status, notes)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(trip_item_id)
        .bind(trip_id)
        .bind(&input.item_id)
        .bind(&input.planned_day)
        .bind(&input.status)
        .bind(&input.notes)
        .execute(&mut connection)
        .await
        .map_err(|error| AppError::database("insert trip item", error))?;

        self.get_trip_item(trip_item_id).await?.ok_or_else(|| {
            AppError::database("load trip item after insert", sqlx::Error::RowNotFound)
        })
    }

    pub async fn list_trip_items(&self, trip_id: &str) -> AppResult<Vec<TripItem>> {
        let mut connection = self.connect().await?;
        let rows = sqlx::query(
            "SELECT
                trip_items.id, trip_items.trip_id, trip_items.item_id, items.name AS item_name,
                trip_items.planned_day, trip_items.status, trip_items.notes
             FROM trip_items
             INNER JOIN items ON items.id = trip_items.item_id
             WHERE trip_items.trip_id = ?
             ORDER BY trip_items.id",
        )
        .bind(trip_id)
        .fetch_all(&mut connection)
        .await
        .map_err(|error| AppError::database("list trip items", error))?;

        rows.into_iter()
            .map(map_trip_item_row)
            .collect::<AppResult<Vec<_>>>()
    }

    pub async fn health_snapshot(&self) -> AppResult<HealthSnapshot> {
        let mut connection = self.connect().await?;

        let item_count = count_rows(&mut connection, "items").await?;
        let location_count = count_rows(&mut connection, "locations").await?;
        let trip_count = count_rows(&mut connection, "trips").await?;

        Ok(HealthSnapshot {
            item_count,
            location_count,
            trip_count,
        })
    }

    async fn connect(&self) -> AppResult<SqliteConnection> {
        db::open_connection(&self.database_file).await
    }

    async fn get_trip_item(&self, id: &str) -> AppResult<Option<TripItem>> {
        let mut connection = self.connect().await?;
        let row = sqlx::query(
            "SELECT
                trip_items.id, trip_items.trip_id, trip_items.item_id, items.name AS item_name,
                trip_items.planned_day, trip_items.status, trip_items.notes
             FROM trip_items
             INNER JOIN items ON items.id = trip_items.item_id
             WHERE trip_items.id = ?",
        )
        .bind(id)
        .fetch_optional(&mut connection)
        .await
        .map_err(|error| AppError::database("get trip item", error))?;

        row.map(map_trip_item_row).transpose()
    }
}

fn map_item_row(row: sqlx::sqlite::SqliteRow) -> AppResult<Item> {
    Ok(Item {
        id: row
            .try_get("id")
            .map_err(|error| AppError::database("read item.id", error))?,
        name: row
            .try_get("name")
            .map_err(|error| AppError::database("read item.name", error))?,
        category: row
            .try_get("category")
            .map_err(|error| AppError::database("read item.category", error))?,
        subcategory: row
            .try_get("subcategory")
            .map_err(|error| AppError::database("read item.subcategory", error))?,
        brand: row
            .try_get("brand")
            .map_err(|error| AppError::database("read item.brand", error))?,
        size: row
            .try_get("size")
            .map_err(|error| AppError::database("read item.size", error))?,
        color_primary: row
            .try_get("color_primary")
            .map_err(|error| AppError::database("read item.color_primary", error))?,
        color_secondary: row
            .try_get("color_secondary")
            .map_err(|error| AppError::database("read item.color_secondary", error))?,
        material: row
            .try_get("material")
            .map_err(|error| AppError::database("read item.material", error))?,
        season: row
            .try_get("season")
            .map_err(|error| AppError::database("read item.season", error))?,
        formality: row
            .try_get("formality")
            .map_err(|error| AppError::database("read item.formality", error))?,
        status: row
            .try_get("status")
            .map_err(|error| AppError::database("read item.status", error))?,
        current_location_id: row
            .try_get("current_location_id")
            .map_err(|error| AppError::database("read item.current_location_id", error))?,
        notes: row
            .try_get("notes")
            .map_err(|error| AppError::database("read item.notes", error))?,
        created_at: row
            .try_get("created_at")
            .map_err(|error| AppError::database("read item.created_at", error))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|error| AppError::database("read item.updated_at", error))?,
    })
}

fn map_location_row(row: sqlx::sqlite::SqliteRow) -> AppResult<Location> {
    Ok(Location {
        id: row
            .try_get("id")
            .map_err(|error| AppError::database("read location.id", error))?,
        name: row
            .try_get("name")
            .map_err(|error| AppError::database("read location.name", error))?,
        location_type: row
            .try_get("location_type")
            .map_err(|error| AppError::database("read location.location_type", error))?,
        parent_id: row
            .try_get("parent_id")
            .map_err(|error| AppError::database("read location.parent_id", error))?,
        notes: row
            .try_get("notes")
            .map_err(|error| AppError::database("read location.notes", error))?,
        created_at: row
            .try_get("created_at")
            .map_err(|error| AppError::database("read location.created_at", error))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|error| AppError::database("read location.updated_at", error))?,
    })
}

fn map_trip_row(row: sqlx::sqlite::SqliteRow) -> AppResult<Trip> {
    Ok(Trip {
        id: row
            .try_get("id")
            .map_err(|error| AppError::database("read trip.id", error))?,
        name: row
            .try_get("name")
            .map_err(|error| AppError::database("read trip.name", error))?,
        destination: row
            .try_get("destination")
            .map_err(|error| AppError::database("read trip.destination", error))?,
        start_date: row
            .try_get("start_date")
            .map_err(|error| AppError::database("read trip.start_date", error))?,
        end_date: row
            .try_get("end_date")
            .map_err(|error| AppError::database("read trip.end_date", error))?,
        trip_type: row
            .try_get("trip_type")
            .map_err(|error| AppError::database("read trip.trip_type", error))?,
        luggage_type: row
            .try_get("luggage_type")
            .map_err(|error| AppError::database("read trip.luggage_type", error))?,
        notes: row
            .try_get("notes")
            .map_err(|error| AppError::database("read trip.notes", error))?,
        created_at: row
            .try_get("created_at")
            .map_err(|error| AppError::database("read trip.created_at", error))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|error| AppError::database("read trip.updated_at", error))?,
    })
}

fn map_movement_row(row: sqlx::sqlite::SqliteRow) -> AppResult<Movement> {
    Ok(Movement {
        id: row
            .try_get("id")
            .map_err(|error| AppError::database("read movement.id", error))?,
        item_id: row
            .try_get("item_id")
            .map_err(|error| AppError::database("read movement.item_id", error))?,
        from_location_id: row
            .try_get("from_location_id")
            .map_err(|error| AppError::database("read movement.from_location_id", error))?,
        to_location_id: row
            .try_get("to_location_id")
            .map_err(|error| AppError::database("read movement.to_location_id", error))?,
        reason: row
            .try_get("reason")
            .map_err(|error| AppError::database("read movement.reason", error))?,
        notes: row
            .try_get("notes")
            .map_err(|error| AppError::database("read movement.notes", error))?,
        moved_at: row
            .try_get("moved_at")
            .map_err(|error| AppError::database("read movement.moved_at", error))?,
    })
}

fn map_trip_item_row(row: sqlx::sqlite::SqliteRow) -> AppResult<TripItem> {
    Ok(TripItem {
        id: row
            .try_get("id")
            .map_err(|error| AppError::database("read trip_item.id", error))?,
        trip_id: row
            .try_get("trip_id")
            .map_err(|error| AppError::database("read trip_item.trip_id", error))?,
        item_id: row
            .try_get("item_id")
            .map_err(|error| AppError::database("read trip_item.item_id", error))?,
        item_name: row
            .try_get("item_name")
            .map_err(|error| AppError::database("read trip_item.item_name", error))?,
        planned_day: row
            .try_get("planned_day")
            .map_err(|error| AppError::database("read trip_item.planned_day", error))?,
        status: row
            .try_get("status")
            .map_err(|error| AppError::database("read trip_item.status", error))?,
        notes: row
            .try_get("notes")
            .map_err(|error| AppError::database("read trip_item.notes", error))?,
    })
}

async fn count_rows(connection: &mut SqliteConnection, table_name: &str) -> AppResult<i64> {
    let query = format!("SELECT COUNT(*) AS count FROM {table_name}");
    sqlx::query_scalar::<_, i64>(&query)
        .fetch_one(connection)
        .await
        .map_err(|error| AppError::database(format!("count rows in {table_name}"), error))
}
