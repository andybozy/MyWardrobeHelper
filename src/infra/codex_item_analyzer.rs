use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use mime_guess::MimeGuess;
use serde::Deserialize;
use tokio::task;

use crate::domain::{AnalyzeItemPhotoInput, ItemPhotoAnalysisSuggestion};
use crate::error::{AppError, AppResult};

const DEFAULT_CODEX_BIN: &str = "codex";
const DEFAULT_CODEX_MODEL: &str = "gpt-5.4";
const OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "name": { "type": ["string", "null"] },
    "category": { "type": ["string", "null"] },
    "subcategory": { "type": ["string", "null"] },
    "brand": { "type": ["string", "null"] },
    "size": { "type": ["string", "null"] },
    "color_primary": { "type": ["string", "null"] },
    "color_secondary": { "type": ["string", "null"] },
    "material": { "type": ["string", "null"] },
    "season": { "type": ["string", "null"] },
    "formality": { "type": ["string", "null"] },
    "status": { "type": ["string", "null"] },
    "notes": { "type": ["string", "null"] },
    "summary": { "type": "string" },
    "warnings": {
      "type": "array",
      "items": { "type": "string" }
    }
  },
  "required": [
    "name",
    "category",
    "subcategory",
    "brand",
    "size",
    "color_primary",
    "color_secondary",
    "material",
    "season",
    "formality",
    "status",
    "notes",
    "summary",
    "warnings"
  ]
}"#;

const ANALYSIS_PROMPT: &str = r#"Analyze the attached photo of a single wardrobe item.

Return only conservative structured suggestions for the item card.

Rules:
- Do not invent details that are not visually grounded.
- If a field is unclear, return null.
- Keep text concise and user-facing.
- `name` should be a short item title if the garment is identifiable.
- `category` should be a broad wardrobe category such as Outerwear, Tops, Bottoms, Footwear, Accessories, Bags, Formalwear, Activewear, Swimwear, Sleepwear, or Other when appropriate.
- `summary` must explain in one short sentence what the model believes is visible.
- `warnings` should contain short strings for ambiguity or uncertainty.
- Return valid JSON matching the provided schema and nothing else in the final answer."#;

#[derive(Debug, Clone)]
pub struct CodexItemAnalyzer {
    command_path: PathBuf,
    work_root: PathBuf,
}

#[derive(Debug, Deserialize)]
struct RawItemPhotoAnalysisSuggestion {
    name: Option<String>,
    category: Option<String>,
    subcategory: Option<String>,
    brand: Option<String>,
    size: Option<String>,
    color_primary: Option<String>,
    color_secondary: Option<String>,
    material: Option<String>,
    season: Option<String>,
    formality: Option<String>,
    status: Option<String>,
    notes: Option<String>,
    summary: String,
    warnings: Vec<String>,
}

impl CodexItemAnalyzer {
    pub fn new(work_root: PathBuf) -> Self {
        Self {
            command_path: PathBuf::from(DEFAULT_CODEX_BIN),
            work_root,
        }
    }

    pub fn with_command(work_root: PathBuf, command_path: PathBuf) -> Self {
        Self {
            command_path,
            work_root,
        }
    }

    pub async fn check_ready(&self) -> AppResult<String> {
        let analyzer = self.clone();
        task::spawn_blocking(move || analyzer.check_ready_blocking())
            .await
            .map_err(|error| AppError::config(format!("wait for codex readiness check: {error}")))?
    }

    pub async fn analyze_upload(
        &self,
        input: AnalyzeItemPhotoInput,
    ) -> AppResult<ItemPhotoAnalysisSuggestion> {
        let analyzer = self.clone();
        task::spawn_blocking(move || analyzer.analyze_upload_blocking(input))
            .await
            .map_err(|error| AppError::config(format!("wait for codex photo analysis: {error}")))?
    }

    pub async fn analyze_path(
        &self,
        image_path: PathBuf,
    ) -> AppResult<ItemPhotoAnalysisSuggestion> {
        let analyzer = self.clone();
        task::spawn_blocking(move || analyzer.analyze_path_blocking(image_path))
            .await
            .map_err(|error| AppError::config(format!("wait for codex photo analysis: {error}")))?
    }

    fn check_ready_blocking(&self) -> AppResult<String> {
        let output = Command::new(&self.command_path)
            .arg("login")
            .arg("status")
            .output()
            .map_err(|error| {
                AppError::io(
                    format!(
                        "run `{}` to check codex login status",
                        self.command_path.display()
                    ),
                    error,
                )
            })?;

        if output.status.success() {
            let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if message.is_empty() {
                "codex login status succeeded".to_string()
            } else {
                message
            })
        } else {
            Err(AppError::config(format!(
                "codex login status failed: {}",
                summarize_process_output(&output.stdout, &output.stderr)
            )))
        }
    }

    fn analyze_upload_blocking(
        &self,
        input: AnalyzeItemPhotoInput,
    ) -> AppResult<ItemPhotoAnalysisSuggestion> {
        if !input.mime_type.starts_with("image/") {
            return Err(AppError::invalid_argument(format!(
                "photo analysis only supports image uploads, got `{}`",
                input.mime_type
            )));
        }

        let run_root = self.prepare_run_root()?;
        let extension = choose_extension(input.original_filename.as_deref(), &input.mime_type);
        let image_path = run_root.join(format!("item-photo.{extension}"));
        fs::write(&image_path, input.bytes).map_err(|error| {
            AppError::io(
                format!("write analysis image {}", image_path.display()),
                error,
            )
        })?;

        self.run_analysis(&run_root, &image_path)
    }

    fn analyze_path_blocking(&self, image_path: PathBuf) -> AppResult<ItemPhotoAnalysisSuggestion> {
        if !image_path.is_file() {
            return Err(AppError::invalid_argument(format!(
                "image path `{}` does not exist",
                image_path.display()
            )));
        }

        let mime_type = MimeGuess::from_path(&image_path)
            .first_raw()
            .unwrap_or("application/octet-stream");
        if !mime_type.starts_with("image/") {
            return Err(AppError::invalid_argument(format!(
                "photo analysis only supports image files, got `{mime_type}`"
            )));
        }

        let run_root = self.prepare_run_root()?;
        self.run_analysis(&run_root, &image_path)
    }

    fn prepare_run_root(&self) -> AppResult<PathBuf> {
        let run_root = self.work_root.join(format!("analysis-{}", unique_suffix()));
        fs::create_dir_all(&run_root).map_err(|error| {
            AppError::io(
                format!("create codex analysis directory {}", run_root.display()),
                error,
            )
        })?;
        Ok(run_root)
    }

    fn run_analysis(
        &self,
        run_root: &Path,
        image_path: &Path,
    ) -> AppResult<ItemPhotoAnalysisSuggestion> {
        let schema_path = run_root.join("schema.json");
        let output_path = run_root.join("result.json");
        fs::write(&schema_path, OUTPUT_SCHEMA).map_err(|error| {
            AppError::io(
                format!("write codex output schema {}", schema_path.display()),
                error,
            )
        })?;

        let output = Command::new(&self.command_path)
            .arg("exec")
            .arg("--ignore-user-config")
            .arg("--ignore-rules")
            .arg("--skip-git-repo-check")
            .arg("--ephemeral")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--color")
            .arg("never")
            .arg("-m")
            .arg(DEFAULT_CODEX_MODEL)
            .arg("--image")
            .arg(image_path)
            .arg("--output-schema")
            .arg(&schema_path)
            .arg("--output-last-message")
            .arg(&output_path)
            .arg(ANALYSIS_PROMPT)
            .current_dir(run_root)
            .output()
            .map_err(|error| {
                AppError::io(
                    format!(
                        "run codex photo analysis via {}",
                        self.command_path.display()
                    ),
                    error,
                )
            })?;

        if !output.status.success() {
            return Err(AppError::config(format!(
                "codex photo analysis failed: {}",
                summarize_process_output(&output.stdout, &output.stderr)
            )));
        }

        let raw = fs::read_to_string(&output_path).map_err(|error| {
            AppError::io(
                format!("read codex analysis result {}", output_path.display()),
                error,
            )
        })?;

        let parsed: RawItemPhotoAnalysisSuggestion = serde_json::from_str(&raw)
            .map_err(|error| AppError::config(format!("parse codex analysis JSON: {error}")))?;

        Ok(normalize_suggestion(parsed))
    }
}

fn normalize_suggestion(raw: RawItemPhotoAnalysisSuggestion) -> ItemPhotoAnalysisSuggestion {
    ItemPhotoAnalysisSuggestion {
        name: normalize_optional(raw.name),
        category: normalize_optional(raw.category),
        subcategory: normalize_optional(raw.subcategory),
        brand: normalize_optional(raw.brand),
        size: normalize_optional(raw.size),
        color_primary: normalize_optional(raw.color_primary),
        color_secondary: normalize_optional(raw.color_secondary),
        material: normalize_optional(raw.material),
        season: normalize_optional(raw.season),
        formality: normalize_optional(raw.formality),
        status: normalize_optional(raw.status),
        notes: normalize_optional(raw.notes),
        summary: normalize_optional(Some(raw.summary))
            .unwrap_or_else(|| "No analysis summary was produced.".to_string()),
        warnings: raw
            .warnings
            .into_iter()
            .filter_map(|warning| normalize_optional(Some(warning)))
            .collect(),
    }
}

fn choose_extension(original_filename: Option<&str>, mime_type: &str) -> String {
    original_filename
        .and_then(|value| Path::new(value).extension())
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_string())
        .or_else(|| {
            mime_type
                .rsplit('/')
                .next()
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "img".to_string())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn summarize_process_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stderr.is_empty() {
        return truncate_message(&stderr);
    }

    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    if !stdout.is_empty() {
        return truncate_message(&stdout);
    }

    "process exited without a detailed error message".to_string()
}

fn truncate_message(message: &str) -> String {
    const LIMIT: usize = 400;
    if message.len() <= LIMIT {
        message.to_string()
    } else {
        format!("{}...", &message[..LIMIT])
    }
}

fn unique_suffix() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    format!("{millis}-{counter}")
}
