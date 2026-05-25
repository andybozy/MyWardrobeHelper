use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::AppConfig;
use crate::error::{AppError, AppResult, io_error, io_error_path};

const PLACEHOLDER_EXPORT_NOTE: &str =
    "Structured wardrobe export will arrive in SEC-017. This file records runtime layout only.";

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupReport {
    pub backup_file: PathBuf,
    pub media_included: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub export_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServePlan {
    pub layout: AppLayout,
    pub bind_url: String,
    pub local_url: String,
    pub lan_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPlan {
    pub layout: AppLayout,
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

    pub fn init(&self) -> AppResult<InitReport> {
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
        touch(&self.database_file)?;

        Ok(InitReport {
            layout: self.clone(),
            created_database_file,
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

pub fn init_app(config: &AppConfig) -> AppResult<InitReport> {
    AppLayout::from_data_dir(config.data_dir.clone()).init()
}

pub fn doctor(config: &AppConfig) -> DoctorReport {
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
        "database file placeholder exists",
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

    checks.push(pass(
        "server_config",
        format!(
            "runtime bind configuration resolved to {} with local URL {}",
            config.bind_url(),
            config.local_url()
        ),
    ));

    checks.push(warn(
        "transport_status",
        "HTTP serve, JSON API, and MCP server are still placeholders until SEC-005 through SEC-007"
            .to_string(),
    ));

    DoctorReport { layout, checks }
}

pub fn plan_serve(config: &AppConfig) -> AppResult<ServePlan> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;

    Ok(ServePlan {
        bind_url: config.bind_url(),
        local_url: config.local_url(),
        lan_url: config.lan_url(),
        layout,
    })
}

pub fn create_backup(config: &AppConfig) -> AppResult<BackupReport> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;

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

pub fn export_layout(config: &AppConfig) -> AppResult<ExportReport> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;

    let export_file = layout
        .exports_root
        .join(format!("wardrobe-layout-{}.json", unix_timestamp_millis()));
    let payload = build_export_payload(&layout);
    fs::write(&export_file, payload).map_err(io_error(format!(
        "write export file {}",
        export_file.display()
    )))?;

    Ok(ExportReport { export_file })
}

pub fn plan_mcp(config: &AppConfig) -> AppResult<McpPlan> {
    let layout = AppLayout::from_data_dir(config.data_dir.clone());
    layout.require_initialized()?;

    Ok(McpPlan { layout })
}

fn build_export_payload(layout: &AppLayout) -> String {
    let generated_at = unix_timestamp_secs();

    format!(
        concat!(
            "{{\n",
            "  \"generated_at_unix_seconds\": {generated_at},\n",
            "  \"data_dir\": \"{data_dir}\",\n",
            "  \"database_file\": \"{database_file}\",\n",
            "  \"item_media_root\": \"{item_media_root}\",\n",
            "  \"backups_root\": \"{backups_root}\",\n",
            "  \"exports_root\": \"{exports_root}\",\n",
            "  \"note\": \"{note}\"\n",
            "}}\n"
        ),
        generated_at = generated_at,
        data_dir = json_escape_path(&layout.root),
        database_file = json_escape_path(&layout.database_file),
        item_media_root = json_escape_path(&layout.item_media_root),
        backups_root = json_escape_path(&layout.backups_root),
        exports_root = json_escape_path(&layout.exports_root),
        note = json_escape(PLACEHOLDER_EXPORT_NOTE),
    )
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

fn warn(label: &'static str, message: String) -> DoctorCheck {
    DoctorCheck {
        status: CheckStatus::Warn,
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

fn json_escape_path(path: &Path) -> String {
    json_escape(&path.display().to_string())
}

fn json_escape(input: &str) -> String {
    input
        .chars()
        .flat_map(|character| match character {
            '\\' => ['\\', '\\'].into_iter().collect::<Vec<_>>(),
            '"' => ['\\', '"'].into_iter().collect::<Vec<_>>(),
            '\n' => ['\\', 'n'].into_iter().collect::<Vec<_>>(),
            '\r' => ['\\', 'r'].into_iter().collect::<Vec<_>>(),
            '\t' => ['\\', 't'].into_iter().collect::<Vec<_>>(),
            other => [other].into_iter().collect::<Vec<_>>(),
        })
        .collect()
}

fn touch(path: &Path) -> AppResult<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(|_| ())
        .map_err(io_error_path("create database file", path))
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

    #[test]
    fn init_creates_expected_layout() {
        let sandbox = TestSandbox::new();
        let config = sandbox.config();

        let report = init_app(&config).expect("init should succeed");

        assert_eq!(report.layout.root, sandbox.data_dir);
        assert!(report.layout.database_file.is_file());
        assert!(report.layout.item_media_root.is_dir());
        assert!(report.layout.backups_root.is_dir());
        assert!(report.layout.exports_root.is_dir());
    }

    #[test]
    fn doctor_reports_missing_initialization() {
        let sandbox = TestSandbox::new();
        let report = doctor(&sandbox.config());

        assert!(report.has_failures());
        assert!(report.checks.iter().any(|check| check.label == "data_dir"));
    }

    #[test]
    fn backup_and_export_require_initialization() {
        let sandbox = TestSandbox::new();
        let backup_error = create_backup(&sandbox.config()).expect_err("backup should fail");
        let export_error = export_layout(&sandbox.config()).expect_err("export should fail");

        assert!(matches!(backup_error, AppError::NotInitialized { .. }));
        assert!(matches!(export_error, AppError::NotInitialized { .. }));
    }

    #[test]
    fn backup_and_export_succeed_after_initialization() {
        let sandbox = TestSandbox::new();
        init_app(&sandbox.config()).expect("init should succeed");

        let backup = create_backup(&sandbox.config()).expect("backup should succeed");
        let export = export_layout(&sandbox.config()).expect("export should succeed");

        assert!(backup.backup_file.is_file());
        assert!(export.export_file.is_file());
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
                "mywardrobehelper-sec002-{}-{}",
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
