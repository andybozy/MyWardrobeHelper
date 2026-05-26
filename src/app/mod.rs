use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::config::AppConfig;
use crate::db;
use crate::domain::{Item, ItemMedia, Location, Movement, PhysicalTag, Trip, TripItem};
use crate::error::{AppError, AppResult, io_error, io_error_path};
use crate::infra::MediaStorage;
use crate::repositories::SqliteWardrobeRepository;
use crate::services::WardrobeService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLayout {
    pub root: PathBuf,
    pub database_file: PathBuf,
    pub media_root: PathBuf,
    pub item_media_root: PathBuf,
    pub backups_root: PathBuf,
    pub exports_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitReport {
    pub layout: AppLayout,
    pub created_database_file: bool,
    pub applied_migration_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReport {
    pub backup_file: PathBuf,
    pub media_included: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub export_file: PathBuf,
    pub item_count: usize,
    pub location_count: usize,
    pub trip_count: usize,
    pub physical_tag_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServePlan {
    pub layout: AppLayout,
    pub bind_url: String,
    pub local_url: String,
    pub lan_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub config: AppConfig,
    pub layout: AppLayout,
    pub service: WardrobeService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub layout: AppLayout,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: CheckStatus,
    pub label: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl AppLayout {
    pub fn from_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        let root = data_dir.into();
        let media_root = root.join("media");
        let item_media_root = media_root.join("items");
        let backups_root = root.join("backups");
        let exports_root = root.join("exports");

        Self {
            database_file: root.join("wardrobe.sqlite3"),
            root,
            media_root,
            item_media_root,
            backups_root,
            exports_root,
        }
    }

    pub async fn init(&self) -> AppResult<InitReport> {
        fs::create_dir_all(&self.root)
            .map_err(io_error_path("create data directory", &self.root))?;
        fs::create_dir_all(&self.item_media_root).map_err(io_error_path(
            "create media directory",
            &self.item_media_root,
        ))?;
        fs::create_dir_all(&self.backups_root).map_err(io_error_path(
            "create backups directory",
            &self.backups_root,
        ))?;
        fs::create_dir_all(&self.exports_root).map_err(io_error_path(
            "create exports directory",
            &self.exports_root,
        ))?;

        let created_database_file = !self.database_file.exists();
        let migration_report = db::migrate_database(&self.database_file).await?;

        Ok(InitReport {
            layout: self.clone(),
            created_database_file,
            applied_migration_count: migration_report.applied_migration_count,
        })
    }

    pub fn require_initialized(&self) -> AppResult<()> {
        if !self.root.is_dir() {
            return Err(AppError::not_initialized(
                &self.root,
                "missing data directory; run `cargo run -- init`",
            ));
        }

        if !self.database_file.is_file() {
            return Err(AppError::not_initialized(
                &self.root,
                format!(
                    "missing database file {}; run `cargo run -- init`",
                    self.database_file.display()
                ),
            ));
        }

        for required_dir in [
            &self.item_media_root,
            &self.backups_root,
            &self.exports_root,
        ] {
            if !required_dir.is_dir() {
                return Err(AppError::not_initialized(
                    &self.root,
                    format!(
                        "missing directory {}; run `cargo run -- init`",
                        required_dir.display()
                    ),
                ));
            }
        }

        Ok(())
    }
}

impl DoctorReport {
    pub fn has_failures(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.status == CheckStatus::Fail)
    }
}

pub async fn init_app(config: &AppConfig) -> AppResult<InitReport> {
    AppLayout::from_data_dir(config.data_dir.clone())
        .init()
        .await
}

pub async fn open_context(config: AppConfig) -> AppResult<AppContext> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;
    ensure_schema_ready(&layout).await?;

    let repository = SqliteWardrobeRepository::new(layout.database_file.clone());
    let service = WardrobeService::new(repository, MediaStorage::new(layout.root.clone()));

    Ok(AppContext {
        config,
        layout,
        service,
    })
}

pub async fn doctor(config: &AppConfig) -> DoctorReport {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    let mut checks = Vec::new();

    if layout.root.is_dir() {
        checks.push(pass(
            "data_dir",
            format!("data directory is ready at {}", layout.root.display()),
        ));
    } else {
        checks.push(fail(
            "data_dir",
            format!(
                "data directory does not exist at {}; run `cargo run -- init`",
                layout.root.display()
            ),
        ));
    }

    checks.push(check_file(
        "database_file",
        &layout.database_file,
        "database file exists",
        "database file is missing; run `cargo run -- init`",
    ));

    checks.push(check_directory(
        "media_dir",
        &layout.item_media_root,
        "item media directory exists",
        "item media directory is missing; run `cargo run -- init`",
    ));

    checks.push(check_directory(
        "backups_dir",
        &layout.backups_root,
        "backups directory exists",
        "backups directory is missing; run `cargo run -- init`",
    ));

    checks.push(check_directory(
        "exports_dir",
        &layout.exports_root,
        "exports directory exists",
        "exports directory is missing; run `cargo run -- init`",
    ));

    if layout.root.is_dir() && layout.database_file.is_file() {
        match OpenOptions::new()
            .read(true)
            .write(true)
            .open(&layout.database_file)
        {
            Ok(_) => checks.push(pass(
                "database_access",
                format!(
                    "database file is readable and writable at {}",
                    layout.database_file.display()
                ),
            )),
            Err(error) => checks.push(fail(
                "database_access",
                format!(
                    "database file is not accessible at {}: {}",
                    layout.database_file.display(),
                    error
                ),
            )),
        }
    } else {
        checks.push(fail(
            "database_access",
            "database file access check skipped because initialization is incomplete".to_string(),
        ));
    }

    for (label, path) in [
        ("media_write", &layout.item_media_root),
        ("backups_write", &layout.backups_root),
        ("exports_write", &layout.exports_root),
    ] {
        checks.push(check_writable_directory(label, path));
    }

    if layout.database_file.is_file() {
        match db::schema_status(&layout.database_file).await {
            Ok(status) if status.missing_tables.is_empty() => checks.push(pass(
                "schema",
                format!(
                    "database schema is ready with {} applied migration(s)",
                    status.applied_migration_count
                ),
            )),
            Ok(status) => checks.push(fail(
                "schema",
                format!(
                    "database schema is incomplete; missing tables: {}. Re-run `cargo run -- init`",
                    status.missing_tables.join(", ")
                ),
            )),
            Err(error) => checks.push(fail(
                "schema",
                format!("database schema check failed: {error}"),
            )),
        }
    } else {
        checks.push(fail(
            "schema",
            "database schema check skipped because the database file is missing".to_string(),
        ));
    }

    match open_context(config.clone()).await {
        Ok(context) => match context.service.health().await {
            Ok(health) => checks.push(pass(
                "service_health",
                format!(
                    "service layer can read wardrobe counts (items: {}, locations: {}, trips: {})",
                    health.item_count, health.location_count, health.trip_count
                ),
            )),
            Err(error) => checks.push(fail(
                "service_health",
                format!("service layer health check failed: {error}"),
            )),
        },
        Err(error) => checks.push(fail(
            "service_health",
            format!("app context could not open the shared service layer: {error}"),
        )),
    }

    checks.push(pass(
        "server_config",
        format!(
            "runtime bind configuration resolved to {} with local URL {}",
            config.bind_url(),
            config.local_url()
        ),
    ));

    checks.push(pass(
        "transport_status",
        "HTTP UI and JSON API are active on this server; embedded MCP is implemented via `mcp serve` or the unified `run` startup."
            .to_string(),
    ));

    DoctorReport { layout, checks }
}

pub async fn plan_serve(config: &AppConfig) -> AppResult<ServePlan> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;
    ensure_schema_ready(&layout).await?;

    Ok(ServePlan {
        bind_url: config.bind_url(),
        local_url: config.local_url(),
        lan_url: config.lan_url(),
        layout,
    })
}

pub async fn create_backup(config: &AppConfig) -> AppResult<BackupReport> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;
    ensure_schema_ready(&layout).await?;

    let backup_file = layout
        .backups_root
        .join(format!("wardrobe-{}.sqlite3", unix_timestamp_millis()));
    fs::copy(&layout.database_file, &backup_file).map_err(io_error(format!(
        "copy database file to backup {}",
        backup_file.display()
    )))?;

    Ok(BackupReport {
        backup_file,
        media_included: false,
    })
}

pub async fn export_layout(config: &AppConfig) -> AppResult<ExportReport> {
    let context = open_context(config.clone()).await?;
    let layout = context.layout.clone();

    let export_file = layout
        .exports_root
        .join(format!("wardrobe-export-{}.json", unix_timestamp_millis()));
    let export_bundle = build_export_bundle(&context).await?;
    let payload = serde_json::to_vec_pretty(&export_bundle)
        .map_err(|error| AppError::config(format!("serialize export JSON: {error}")))?;
    fs::write(&export_file, payload).map_err(io_error(format!(
        "write export file {}",
        export_file.display()
    )))?;

    Ok(ExportReport {
        export_file,
        item_count: export_bundle.items.len(),
        location_count: export_bundle.locations.len(),
        trip_count: export_bundle.trips.len(),
        physical_tag_count: export_bundle.physical_tags.len(),
    })
}

#[derive(Debug, Serialize)]
struct ExportBundle {
    generated_at_unix_seconds: u64,
    data_dir: String,
    database_file: String,
    media_root: String,
    notes: ExportNotes,
    items: Vec<Item>,
    item_media: Vec<ItemMedia>,
    locations: Vec<Location>,
    movements: Vec<Movement>,
    trips: Vec<Trip>,
    trip_items: Vec<TripItem>,
    physical_tags: Vec<PhysicalTag>,
}

#[derive(Debug, Serialize)]
struct ExportNotes {
    media_files_included: bool,
    media_files_description: &'static str,
}

async fn build_export_bundle(context: &AppContext) -> AppResult<ExportBundle> {
    let items = context.service.list_items().await?;
    let locations = context.service.list_locations().await?;
    let trips = context.service.list_trips().await?;
    let physical_tags = context.service.list_physical_tags().await?;

    let mut item_media = Vec::new();
    let mut movements = Vec::new();
    for item in &items {
        item_media.extend(context.service.list_item_media(&item.id).await?);
        movements.extend(context.service.get_item_movements(&item.id).await?);
    }

    let mut trip_items = Vec::new();
    for trip in &trips {
        trip_items.extend(context.service.list_trip_items(&trip.id).await?);
    }

    Ok(ExportBundle {
        generated_at_unix_seconds: unix_timestamp_secs(),
        data_dir: context.layout.root.display().to_string(),
        database_file: context.layout.database_file.display().to_string(),
        media_root: context.layout.media_root.display().to_string(),
        notes: ExportNotes {
            media_files_included: false,
            media_files_description: "This export includes media metadata only. Media files remain on disk under media/items/.",
        },
        items,
        item_media,
        locations,
        movements,
        trips,
        trip_items,
        physical_tags,
    })
}

fn check_file(
    label: &'static str,
    path: &Path,
    success_message: &str,
    failure_message: &str,
) -> DoctorCheck {
    if path.is_file() {
        pass(label, format!("{success_message} at {}", path.display()))
    } else {
        fail(label, failure_message.to_string())
    }
}

fn check_directory(
    label: &'static str,
    path: &Path,
    success_message: &str,
    failure_message: &str,
) -> DoctorCheck {
    if path.is_dir() {
        pass(label, format!("{success_message} at {}", path.display()))
    } else {
        fail(label, failure_message.to_string())
    }
}

fn check_writable_directory(label: &'static str, path: &Path) -> DoctorCheck {
    if !path.is_dir() {
        return fail(
            label,
            format!(
                "write check skipped because directory does not exist at {}",
                path.display()
            ),
        );
    }

    let marker = path.join(".write-check");
    match fs::File::create(&marker) {
        Ok(mut file) => {
            if let Err(error) = file.write_all(b"ok") {
                let _ = fs::remove_file(&marker);
                return fail(
                    label,
                    format!("directory is not writable at {}: {}", path.display(), error),
                );
            }

            let _ = fs::remove_file(&marker);
            pass(
                label,
                format!("directory is writable at {}", path.display()),
            )
        }
        Err(error) => fail(
            label,
            format!("directory is not writable at {}: {}", path.display(), error),
        ),
    }
}

fn pass(label: &'static str, message: String) -> DoctorCheck {
    DoctorCheck {
        status: CheckStatus::Pass,
        label,
        message,
    }
}

fn fail(label: &'static str, message: String) -> DoctorCheck {
    DoctorCheck {
        status: CheckStatus::Fail,
        label,
        message,
    }
}

async fn ensure_schema_ready(layout: &AppLayout) -> AppResult<()> {
    let status = db::schema_status(&layout.database_file).await?;
    if status.missing_tables.is_empty() {
        Ok(())
    } else {
        Err(AppError::not_initialized(
            &layout.root,
            format!(
                "database schema is incomplete; missing tables: {}. Run `cargo run -- init`",
                status.missing_tables.join(", ")
            ),
        ))
    }
}

fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn unix_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, DEFAULT_HOST, DEFAULT_PORT};
    use std::env;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[tokio::test]
    async fn init_creates_expected_layout() {
        let sandbox = TestSandbox::new();
        let config = sandbox.config();

        let report = init_app(&config).await.expect("init should succeed");

        assert_eq!(report.layout.root, sandbox.data_dir);
        assert!(report.layout.database_file.is_file());
        assert!(report.layout.item_media_root.is_dir());
        assert!(report.layout.backups_root.is_dir());
        assert!(report.layout.exports_root.is_dir());
        assert!(report.applied_migration_count >= 1);
    }

    #[tokio::test]
    async fn doctor_reports_missing_initialization() {
        let sandbox = TestSandbox::new();
        let report = doctor(&sandbox.config()).await;

        assert!(report.has_failures());
        assert!(report.checks.iter().any(|check| check.label == "data_dir"));
    }

    #[tokio::test]
    async fn doctor_reports_current_transport_surface() {
        let sandbox = TestSandbox::new();
        let config = sandbox.config();
        init_app(&config).await.expect("init should succeed");

        let report = doctor(&config).await;
        let transport = report
            .checks
            .iter()
            .find(|check| check.label == "transport_status")
            .expect("transport status check should exist");

        assert_eq!(transport.status, CheckStatus::Pass);
        assert!(
            transport
                .message
                .contains("HTTP UI and JSON API are active")
        );
        assert!(transport.message.contains("mcp serve"));
        assert!(transport.message.contains("run"));
    }

    #[tokio::test]
    async fn backup_and_export_require_initialization() {
        let sandbox = TestSandbox::new();
        let backup_error = create_backup(&sandbox.config())
            .await
            .expect_err("backup should fail");
        let export_error = export_layout(&sandbox.config())
            .await
            .expect_err("export should fail");

        assert!(matches!(backup_error, AppError::NotInitialized { .. }));
        assert!(matches!(export_error, AppError::NotInitialized { .. }));
    }

    #[tokio::test]
    async fn backup_and_export_succeed_after_initialization() {
        let sandbox = TestSandbox::new();
        init_app(&sandbox.config())
            .await
            .expect("init should succeed");

        let backup = create_backup(&sandbox.config())
            .await
            .expect("backup should succeed");
        let export = export_layout(&sandbox.config())
            .await
            .expect("export should succeed");

        assert!(backup.backup_file.is_file());
        assert!(export.export_file.is_file());
    }

    #[tokio::test]
    async fn export_contains_structured_records() {
        let sandbox = TestSandbox::new();
        let config = sandbox.config();
        init_app(&config).await.expect("init should succeed");

        let context = open_context(config.clone()).await.expect("open context");
        let item = context
            .service
            .create_item(crate::domain::NewItem {
                name: "Export Blazer".to_string(),
                category: Some("Outerwear".to_string()),
                subcategory: None,
                brand: None,
                size: None,
                color_primary: None,
                color_secondary: None,
                material: None,
                season: Some("Summer".to_string()),
                formality: None,
                status: Some("ready".to_string()),
                current_location_id: None,
                notes: None,
            })
            .await
            .expect("create item");
        let _ = context
            .service
            .register_physical_tag(crate::domain::NewPhysicalTag {
                tag_type: "nfc".to_string(),
                external_identifier: "export-tag-001".to_string(),
                label: Some("Export Tag".to_string()),
                bound_entity_type: "item".to_string(),
                bound_entity_id: item.id,
                notes: None,
            })
            .await
            .expect("register tag");

        let report = export_layout(&config).await.expect("export should succeed");
        let payload = fs::read_to_string(&report.export_file).expect("read export file");

        assert!(payload.contains("\"items\""));
        assert!(payload.contains("\"physical_tags\""));
        assert!(payload.contains("Export Blazer"));
        assert!(payload.contains("export-tag-001"));
    }

    struct TestSandbox {
        root: PathBuf,
        data_dir: PathBuf,
    }

    impl TestSandbox {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);

            let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = env::temp_dir().join(format!(
                "mywardrobehelper-sec003-{}-{}",
                std::process::id(),
                unique
            ));

            Self {
                data_dir: root.join("data"),
                root,
            }
        }

        fn config(&self) -> AppConfig {
            AppConfig {
                host: DEFAULT_HOST.to_string(),
                port: DEFAULT_PORT,
                data_dir: self.data_dir.clone(),
            }
        }
    }

    impl Drop for TestSandbox {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
