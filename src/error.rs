use deadpool_postgres::PoolError;
use thiserror::Error;
use tokio_postgres::Error as TokioPostgresError;

use crate::auth::error::EmailError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration errors: {}", .0.join(", "))]
    Config(Vec<String>),

    #[error("Authentication/Authorization error: {0}")]
    Auth(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    /// External service error with raw response body (e.g. OpenRouter) for debugging.
    #[error("External service error: {message}")]
    ExternalServiceWithRaw {
        message: String,
        raw_response: String,
    },

    /// Transient external service error (e.g. 500, error body); safe to retry.
    #[error("External service error (retryable): {message}")]
    ExternalServiceRetryable {
        message: String,
        raw_response: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            AppError::Database(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Migration(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Auth(_) => actix_web::http::StatusCode::FORBIDDEN, // Or FORBIDDEN
            AppError::NotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            AppError::ExternalService(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceWithRaw { .. } => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::ExternalServiceRetryable { .. } => {
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::Validation(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::BadRequest(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Unauthorized(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            AppError::Internal(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Reqwest(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Json(_) => actix_web::http::StatusCode::BAD_REQUEST,
            AppError::Redis(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => actix_web::http::StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let err_str = self.to_string();
        let mut body = serde_json::json!({
            "error": err_str,
            "message": err_str,
        });
        if let AppError::ExternalServiceWithRaw { raw_response, .. }
        | AppError::ExternalServiceRetryable { raw_response, .. } = self
        {
            body["raw_response"] = serde_json::Value::String(raw_response.clone());
        }
        actix_web::HttpResponse::build(self.status_code()).json(body)
    }
}

// Type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

// Implementation for VarError
impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::Config(vec![format!("Environment variable error: {}", err)])
    }
}

// Implementation for ParseIntError
impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::Config(vec![format!("Configuration parsing error: {}", err)])
    }
}

// Implementation for tokio_postgres::Error
impl From<TokioPostgresError> for AppError {
    fn from(err: TokioPostgresError) -> Self {
        AppError::Database(err.to_string())
    }
}

// Implementation for deadpool_postgres::PoolError
impl From<PoolError> for AppError {
    fn from(err: PoolError) -> Self {
        AppError::Database(format!("DB Pool error: {}", err))
    }
}

// Implementation for deadpool_postgres::ConfigError
impl From<deadpool_postgres::ConfigError> for AppError {
    fn from(err: deadpool_postgres::ConfigError) -> Self {
        AppError::Config(vec![format!("DB config error: {}", err)])
    }
}

// Implementation for deadpool::managed::PoolError
impl From<deadpool::managed::PoolError<deadpool_postgres::ConfigError>> for AppError {
    fn from(err: deadpool::managed::PoolError<deadpool_postgres::ConfigError>) -> Self {
        AppError::Config(vec![format!("DB pool error: {}", err)])
    }
}

// Implementation for auth::error::EmailError
impl From<EmailError> for AppError {
    fn from(err: EmailError) -> Self {
        AppError::ExternalService(format!("Email service error: {}", err))
    }
}
