use std::fmt;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    UnknownSite,
    MissingDatabaseCredentials,
    IoError(std::io::Error),
    BackupError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::UnknownSite => write!(f, "Unknown site type"),
            AppError::MissingDatabaseCredentials => write!(f, "Missing database credentials"),
            AppError::IoError(err) => write!(f, "IO Error: {}", err),
            AppError::BackupError(err) => write!(f, "Error executing backup: {}", err),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IoError(value)
    }
}
