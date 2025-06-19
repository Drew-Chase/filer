use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use std::io;
use std::path::PathBuf;

/// Comprehensive error type for the application
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// General internal server error
    #[error("An internal server error occurred: {0}")]
    InternalError(#[from] anyhow::Error),

    /// File system related errors
    #[error("File system error: {message}")]
    FilesystemError {
        message: String,
        #[source]
        source: Option<io::Error>,
        path: Option<PathBuf>,
    },

    /// File not found error
    #[error("File not found: {path}")]
    NotFound {
        path: String,
    },

    /// Permission denied error
    #[error("Permission denied: {path}")]
    PermissionDenied {
        path: String,
    },

    /// Invalid input error
    #[error("Invalid input: {message}")]
    InvalidInput {
        message: String,
    },

    /// Authentication error
    #[error("Authentication error: {message}")]
    AuthenticationError {
        message: String,
    },

    /// Authorization error
    #[error("Authorization error: {message}")]
    AuthorizationError {
        message: String,
    },

    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        field: Option<String>,
    },

    /// Rate limit exceeded error
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Database error
    #[error("Database error: {message}")]
    DatabaseError {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::FilesystemError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            Self::InvalidInput { .. } => StatusCode::BAD_REQUEST,
            Self::AuthenticationError { .. } => StatusCode::UNAUTHORIZED,
            Self::AuthorizationError { .. } => StatusCode::FORBIDDEN,
            Self::ValidationError { .. } => StatusCode::BAD_REQUEST,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_message = self.to_string();

        // Create a JSON response with error details
        let json_body = match self {
            Self::NotFound { path } => {
                json!({
                    "error": "not_found",
                    "message": error_message,
                    "path": path
                })
            },
            Self::PermissionDenied { path } => {
                json!({
                    "error": "permission_denied",
                    "message": error_message,
                    "path": path
                })
            },
            Self::ValidationError { message, field } => {
                json!({
                    "error": "validation_error",
                    "message": message,
                    "field": field
                })
            },
            _ => {
                json!({
                    "error": status.canonical_reason().unwrap_or("error"),
                    "message": error_message
                })
            }
        };

        HttpResponse::build(status).json(json_body)
    }
}

// Implement From traits for common error types
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::NotFound { 
                path: err.to_string() 
            },
            io::ErrorKind::PermissionDenied => Self::PermissionDenied { 
                path: err.to_string() 
            },
            _ => Self::FilesystemError { 
                message: err.to_string(), 
                source: Some(err),
                path: None,
            }
        }
    }
}

// Helper functions to create specific errors
impl Error {
    pub fn filesystem_error<S: Into<String>>(message: S, source: Option<io::Error>, path: Option<PathBuf>) -> Self {
        Self::FilesystemError {
            message: message.into(),
            source,
            path,
        }
    }

    pub fn not_found<S: Into<String>>(path: S) -> Self {
        Self::NotFound {
            path: path.into(),
        }
    }

    pub fn permission_denied<S: Into<String>>(path: S) -> Self {
        Self::PermissionDenied {
            path: path.into(),
        }
    }

    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    pub fn validation_error<S: Into<String>, F: Into<String>>(message: S, field: Option<F>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: field.map(|f| f.into()),
        }
    }

    pub fn authentication_error<S: Into<String>>(message: S) -> Self {
        Self::AuthenticationError {
            message: message.into(),
        }
    }

    pub fn authorization_error<S: Into<String>>(message: S) -> Self {
        Self::AuthorizationError {
            message: message.into(),
        }
    }

    pub fn database_error<S: Into<String>>(message: S, source: Option<anyhow::Error>) -> Self {
        Self::DatabaseError {
            message: message.into(),
            source,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
