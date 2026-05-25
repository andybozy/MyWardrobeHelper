use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, Row, SqliteConnection};

const EXPECTED_TABLES: &[&str] = &[
    "items",
    "item_media",
    "locations",
    "movements",
    "trips",
    "trip_items",
    "physical_tags",
];

#[tokio::test]
async fn init_command_creates_expected_schema_tables() {
    let sandbox = TestSandbox::new();
    let output = Command::new(env!("CARGO_BIN_EXE_mywardrobehelper"))
        .args(["init", "--data-dir"])
        .arg(&sandbox.data_dir)
        .output()
        .expect("run init command");

    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let database_file = sandbox.data_dir.join("wardrobe.sqlite3");
    assert!(database_file.is_file(), "database file was not created");

    let mut connection =
        SqliteConnection::connect_with(&SqliteConnectOptions::new().filename(&database_file))
            .await
            .expect("connect to initialized sqlite database");

    let rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&mut connection)
    .await
    .expect("query sqlite schema tables");

    let table_names: Vec<String> = rows
        .into_iter()
        .map(|row| row.try_get::<String, _>("name").expect("read table name"))
        .collect();

    for expected in EXPECTED_TABLES {
        assert!(
            table_names.iter().any(|table| table == expected),
            "missing expected table {expected}; tables: {table_names:?}"
        );
    }
}

#[test]
fn serve_guides_clearly_when_data_dir_is_missing() {
    let sandbox = TestSandbox::new();
    let output = Command::new(env!("CARGO_BIN_EXE_mywardrobehelper"))
        .args(["serve", "--data-dir"])
        .arg(&sandbox.data_dir)
        .output()
        .expect("run serve command");

    assert!(!output.status.success(), "serve should fail before init");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("run `cargo run -- init`"),
        "serve error should guide the user clearly, got: {stderr}"
    );
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
            "mywardrobehelper-schema-test-{}-{}",
            std::process::id(),
            unique
        ));

        Self {
            data_dir: root.join("data"),
            root,
        }
    }
}

impl Drop for TestSandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
