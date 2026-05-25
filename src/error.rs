use std::fmt::{Display, Formatter};
use std::io;
use std::path::{Path, PathBuf};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    InvalidArgument(String),
    Config(String),
    Io { context: String, source: io::Error },
    NotInitialized { data_dir: PathBuf, reason: String },
}

impl AppError {
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn io(context: impl Into<String>, source: io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }

    pub fn not_initialized(data_dir: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::NotInitialized {
            data_dir: data_dir.into(),
            reason: reason.into(),
        }
    }
}

pub fn io_error(context: impl Into<String>) -> impl FnOnce(io::Error) -> AppError {
    move |source| AppError::io(context.into(), source)
}

pub fn io_error_path(action: &str, path: &Path) -> impl FnOnce(io::Error) -> AppError {
    let context = format!("{action} {}", path.display());
    move |source| AppError::io(context, source)
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::Config(message) => write!(f, "configuration error: {message}"),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
            Self::NotInitialized { data_dir, reason } => {
                write!(
                    f,
                    "data directory {} is not initialized: {reason}",
                    data_dir.display()
                )
            }
        }
    }
}

impl std::error::Error for AppError {}
