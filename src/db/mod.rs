use std::collections::BTreeSet;
use std::path::Path;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Connection, Row, SqliteConnection};

use crate::error::{AppError, AppResult};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

const EXPECTED_TABLES: &[&str] = &[
    "items",
    "item_media",
    "locations",
    "movements",
    "trips",
    "trip_items",
    "physical_tags",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub applied_migration_count: usize,
    pub known_tables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaStatus {
    pub applied_migration_count: usize,
    pub missing_tables: Vec<String>,
    pub known_tables: Vec<String>,
}

pub async fn migrate_database(database_file: &Path) -> AppResult<MigrationReport> {
    let mut connection = connect(database_file, true).await?;
    MIGRATOR
        .run(&mut connection)
        .await
        .map_err(|error| AppError::database("run SQLite migrations", error.into()))?;

    let status = schema_status_with_connection(&mut connection).await?;
    Ok(MigrationReport {
        applied_migration_count: status.applied_migration_count,
        known_tables: status.known_tables,
    })
}

pub async fn schema_status(database_file: &Path) -> AppResult<SchemaStatus> {
    let mut connection = connect(database_file, false).await?;
    schema_status_with_connection(&mut connection).await
}

async fn schema_status_with_connection(
    connection: &mut SqliteConnection,
) -> AppResult<SchemaStatus> {
    let rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&mut *connection)
    .await
    .map_err(|error| AppError::database("query SQLite schema tables", error))?;

    let known_tables: Vec<String> = rows
        .into_iter()
        .map(|row| row.try_get::<String, _>("name"))
        .collect::<Result<_, _>>()
        .map_err(|error| AppError::database("read SQLite schema table names", error))?;

    let known_table_set: BTreeSet<&str> = known_tables.iter().map(String::as_str).collect();
    let missing_tables = EXPECTED_TABLES
        .iter()
        .filter(|table| !known_table_set.contains(**table))
        .map(|table| (*table).to_string())
        .collect();

    let applied_migration_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&mut *connection)
            .await
            .map_err(|error| AppError::database("count applied SQLite migrations", error))?;

    Ok(SchemaStatus {
        applied_migration_count: applied_migration_count as usize,
        missing_tables,
        known_tables,
    })
}

async fn connect(database_file: &Path, create_if_missing: bool) -> AppResult<SqliteConnection> {
    let options = SqliteConnectOptions::new()
        .filename(database_file)
        .create_if_missing(create_if_missing)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);

    SqliteConnection::connect_with(&options)
        .await
        .map_err(|error| {
            AppError::database(
                format!("open SQLite database {}", database_file.display()),
                error,
            )
        })
}
