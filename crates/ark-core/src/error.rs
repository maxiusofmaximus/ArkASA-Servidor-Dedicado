use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid {field}: {message} (code: {code})")]
    ValidationFailed {
        field: String,
        message: String,
        code: String,
    },

    #[error("Server not found at {path}")]
    ServerNotFound { path: String },

    #[error("Config file not found: {path}")]
    ConfigNotFound { path: String },

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Path error: {path} is invalid or inaccessible")]
    PathError { path: String },

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl Error {
    pub fn field_error(field: &str, message: &str, code: &str) -> Self {
        Self::ValidationFailed {
            field: field.to_string(),
            message: message.to_string(),
            code: code.to_string(),
        }
    }

    pub fn to_validation_error(&self) -> ValidationError {
        match self {
            Error::ValidationFailed { field, message, code } => ValidationError {
                field: field.clone(),
                message: message.clone(),
                code: code.clone(),
            },
            _ => ValidationError {
                field: "unknown".to_string(),
                message: self.to_string(),
                code: "UNKNOWN".to_string(),
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
