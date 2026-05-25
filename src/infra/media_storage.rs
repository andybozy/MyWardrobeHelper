use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct MediaStorage {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredMediaFile {
    pub relative_file_path: String,
    pub original_filename: String,
    pub file_size_bytes: i64,
}

impl MediaStorage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub async fn store_item_media(
        &self,
        item_id: &str,
        original_filename: Option<&str>,
        bytes: &[u8],
    ) -> AppResult<StoredMediaFile> {
        let safe_original = sanitize_filename(original_filename.unwrap_or("upload.bin"));
        let extension = Path::new(&safe_original)
            .extension()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .unwrap_or("bin");
        let stored_filename = format!("{}.{extension}", unique_stem());
        let item_dir = self.data_dir.join("media").join("items").join(item_id);

        tokio::fs::create_dir_all(&item_dir)
            .await
            .map_err(|error| {
                AppError::io(
                    format!("create item media directory {}", item_dir.display()),
                    error,
                )
            })?;

        let absolute_path = item_dir.join(&stored_filename);
        tokio::fs::write(&absolute_path, bytes)
            .await
            .map_err(|error| {
                AppError::io(
                    format!("write media file {}", absolute_path.display()),
                    error,
                )
            })?;

        Ok(StoredMediaFile {
            relative_file_path: format!("media/items/{item_id}/{stored_filename}"),
            original_filename: safe_original,
            file_size_bytes: bytes.len() as i64,
        })
    }

    pub async fn read_relative(&self, relative_file_path: &str) -> AppResult<Vec<u8>> {
        let relative_path = validate_relative_path(relative_file_path)?;
        let absolute_path = self.data_dir.join(relative_path);
        tokio::fs::read(&absolute_path).await.map_err(|error| {
            AppError::io(
                format!("read media file {}", absolute_path.display()),
                error,
            )
        })
    }

    pub fn absolute_path(&self, relative_file_path: &str) -> AppResult<PathBuf> {
        let relative_path = validate_relative_path(relative_file_path)?;
        Ok(self.data_dir.join(relative_path))
    }
}

fn validate_relative_path(relative_file_path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(relative_file_path);
    if path.is_absolute() {
        return Err(AppError::invalid_argument(
            "media path must stay relative to the data directory",
        ));
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AppError::invalid_argument(
            "media path must not contain parent directory segments",
        ));
    }

    Ok(path)
}

fn sanitize_filename(input: &str) -> String {
    let mut sanitized = String::with_capacity(input.len());

    for character in input.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
            sanitized.push(character);
        } else if !character.is_whitespace() {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() {
        "upload.bin".to_string()
    } else {
        sanitized
    }
}

fn unique_stem() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("media-{millis}-{counter}")
}
