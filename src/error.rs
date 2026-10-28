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

    /// The write conflicts with existing data (e.g. a unique constraint).
    #[error("Conflict: {0}")]
    Conflict(String),

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
            AppError::Conflict(_) => actix_web::http::StatusCode::CONFLICT,
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
        let status = self.status_code();
        if status.is_server_error() {
            // Internal detail (SQL, upstream bodies) goes to the log, never to the client.
            let error_id = new_error_id();
            match self {
                AppError::ExternalServiceWithRaw { raw_response, .. }
                | AppError::ExternalServiceRetryable { raw_response, .. } => {
                    log::error!("[{error_id}] {self}; raw response: {raw_response}")
                }
                _ => log::error!("[{error_id}] {self}"),
            }
            return internal_error_response(status, &error_id);
        }
        let err_str = self.to_string();
        actix_web::HttpResponse::build(status).json(serde_json::json!({
            "error": err_str,
            "message": err_str,
            "code": self.code(),
        }))
    }
}

impl AppError {
    /// Stable machine-readable code sent to clients alongside the message.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Auth(_) => "forbidden",
            AppError::NotFound(_) => "not_found",
            AppError::Validation(_) | AppError::BadRequest(_) | AppError::Json(_) => "bad_request",
            AppError::Conflict(_) => "conflict",
            AppError::Unauthorized(_) | AppError::Jwt(_) => "unauthorized",
            AppError::Database(_)
            | AppError::Migration(_)
            | AppError::Io(_)
            | AppError::Config(_)
            | AppError::ExternalService(_)
            | AppError::ExternalServiceWithRaw { .. }
            | AppError::ExternalServiceRetryable { .. }
            | AppError::Internal(_)
            | AppError::Reqwest(_)
            | AppError::Redis(_) => "internal_error",
        }
    }
}

impl AppError {
    /// The message without the variant prefix that `Display` adds. For errors built
    /// from a message this is that message; wrapped errors give their own text. Lets
    /// handlers that were written against `Box<dyn Error>` keep the same bodies.
    pub fn detail(&self) -> String {
        match self {
            AppError::Database(s)
            | AppError::Migration(s)
            | AppError::Auth(s)
            | AppError::NotFound(s)
            | AppError::ExternalService(s)
            | AppError::Validation(s)
            | AppError::Conflict(s)
            | AppError::BadRequest(s)
            | AppError::Unauthorized(s)
            | AppError::Internal(s) => s.clone(),
            AppError::ExternalServiceWithRaw { message, .. }
            | AppError::ExternalServiceRetryable { message, .. } => message.clone(),
            AppError::Config(v) => v.join(", "),
            AppError::Io(e) => e.to_string(),
            AppError::Reqwest(e) => e.to_string(),
            AppError::Json(e) => e.to_string(),
            AppError::Redis(e) => e.to_string(),
            AppError::Jwt(e) => e.to_string(),
        }
    }

    /// True for the typed client errors that the database mapping produces
    /// (constraint conflicts and rejected input).
    pub fn is_input_error(&self) -> bool {
        matches!(self, AppError::Conflict(_) | AppError::Validation(_))
    }
}

/// For handlers that answer every service error with their own 5xx body: typed
/// input errors (409/400 from constraint violations) keep their typed response,
/// anything else gets `fallback`.
pub fn input_error_or(
    e: AppError,
    fallback: impl FnOnce(AppError) -> actix_web::HttpResponse,
) -> actix_web::HttpResponse {
    if e.is_input_error() {
        actix_web::ResponseError::error_response(&e)
    } else {
        fallback(e)
    }
}

/// Message shown to clients for every 5xx response.
pub const INTERNAL_ERROR_MESSAGE: &str = "Internal server error";

/// Marks a 5xx response whose body is already safe, so
/// [`crate::middleware::error_sanitizer`] passes it through.
#[derive(Clone, Copy, Debug)]
pub struct SanitizedServerError;

/// Short id that ties a client-visible 5xx to its server log line.
pub fn new_error_id() -> String {
    let mut id = uuid::Uuid::new_v4().simple().to_string();
    id.truncate(12);
    id
}

/// Generic 5xx body: `{error, message, code: "internal_error", error_id}`.
pub fn internal_error_response(
    status: actix_web::http::StatusCode,
    error_id: &str,
) -> actix_web::HttpResponse {
    let mut response = actix_web::HttpResponse::build(status).json(serde_json::json!({
        "error": INTERNAL_ERROR_MESSAGE,
        "message": INTERNAL_ERROR_MESSAGE,
        "code": "internal_error",
        "error_id": error_id,
    }));
    response.extensions_mut().insert(SanitizedServerError);
    response
}

/// Client errors that Postgres reports for bad or conflicting input.
fn client_error_for_sql_state(state: &tokio_postgres::error::SqlState) -> Option<AppError> {
    use tokio_postgres::error::SqlState;
    if *state == SqlState::UNIQUE_VIOLATION {
        Some(AppError::Conflict(
            "A record with these values already exists".to_string(),
        ))
    } else if *state == SqlState::FOREIGN_KEY_VIOLATION {
        Some(AppError::Conflict(
            "The referenced record does not exist or is still in use".to_string(),
        ))
    } else if *state == SqlState::CHECK_VIOLATION
        || *state == SqlState::NOT_NULL_VIOLATION
        || *state == SqlState::INVALID_TEXT_REPRESENTATION
        || *state == SqlState::STRING_DATA_RIGHT_TRUNCATION
        || *state == SqlState::NUMERIC_VALUE_OUT_OF_RANGE
    {
        Some(AppError::Validation("Invalid input".to_string()))
    } else {
        None
    }
}

// Type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Ad-hoc message errors (`Err("...")?`, `.ok_or("...")?`, `format!(..)` strings) that
/// used to become `Box<dyn Error>`. The variant is picked from the text the same way the
/// migration did; `detail()` returns the message unchanged, so text-matching handlers and
/// response bodies built from `detail()` keep working.
pub fn from_message(msg: String) -> AppError {
    let m = msg.to_lowercase();
    if m.contains("not found") || m.contains("does not exist") {
        AppError::NotFound(msg)
    } else if [
        "unauthorized",
        "access denied",
        "permission",
        "not allowed",
        "forbidden",
        "only delete your own",
        "only edit your own",
        "not the owner",
    ]
    .iter()
    .any(|k| m.contains(k))
    {
        AppError::Auth(msg)
    } else if m.starts_with("failed") || m.contains("internal") {
        AppError::Internal(msg)
    } else {
        AppError::BadRequest(msg)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        from_message(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        from_message(msg.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<fancy_regex::Error> for AppError {
    fn from(e: fancy_regex::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<mailparse::MailParseError> for AppError {
    fn from(e: mailparse::MailParseError) -> Self {
        AppError::BadRequest(e.to_string())
    }
}

impl From<std::path::StripPrefixError> for AppError {
    fn from(e: std::path::StripPrefixError) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<crate::language::models::MathJaxValidationError> for AppError {
    fn from(e: crate::language::models::MathJaxValidationError) -> Self {
        AppError::Validation(e.to_string())
    }
}

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
        if let Some(client_error) = err.code().and_then(client_error_for_sql_state) {
            log::info!("database rejected input: {err:?}");
            return client_error;
        }
        if err.code() == Some(&tokio_postgres::error::SqlState::NO_DATA_FOUND) {
            // P0002, raised by our plpgsql lookups when a row is missing.
            let msg = err
                .as_db_error()
                .map(|db| db.message().to_string())
                .unwrap_or_else(|| err.to_string());
            return AppError::NotFound(msg);
        }
        match err.as_db_error() {
            Some(db) => AppError::Database(format!("{err}: {}", db.message())),
            None => AppError::Database(err.to_string()),
        }
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

#[cfg(test)]
mod tests {
    use actix_web::{body::MessageBody, http::StatusCode, ResponseError};
    use tokio_postgres::error::SqlState;

    use super::{client_error_for_sql_state, AppError};

    #[test]
    fn detail_drops_variant_prefix() {
        assert_eq!(
            AppError::NotFound("Comment not found".into()).detail(),
            "Comment not found"
        );
        assert_eq!(
            AppError::Auth("Access denied".into()).to_string(),
            "Authentication/Authorization error: Access denied"
        );
        assert_eq!(
            AppError::Auth("Access denied".into()).detail(),
            "Access denied"
        );
    }

    #[test]
    fn input_error_or_keeps_typed_conflict() {
        let fallback = |_e: AppError| actix_web::HttpResponse::InternalServerError().finish();
        let r = super::input_error_or(AppError::Conflict("dup".into()), fallback);
        assert_eq!(r.status(), StatusCode::CONFLICT);
        let r = super::input_error_or(AppError::Validation("Invalid input".into()), fallback);
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
        let r = super::input_error_or(AppError::BadRequest("x".into()), fallback);
        assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let r = super::input_error_or(AppError::Database("x".into()), fallback);
        assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn sql_states_for_bad_input_become_client_errors() {
        let conflict = client_error_for_sql_state(&SqlState::UNIQUE_VIOLATION);
        assert!(matches!(conflict, Some(AppError::Conflict(_))));
        let invalid = client_error_for_sql_state(&SqlState::CHECK_VIOLATION);
        assert!(matches!(invalid, Some(AppError::Validation(_))));
        assert!(client_error_for_sql_state(&SqlState::UNDEFINED_TABLE).is_none());
    }

    #[test]
    fn server_error_body_hides_internal_text() {
        for error in [
            AppError::Database("relation \"secret_table\" does not exist".to_string()),
            AppError::Internal("/srv/secret/path".to_string()),
            AppError::ExternalServiceWithRaw {
                message: "upstream".to_string(),
                raw_response: "secret upstream body".to_string(),
            },
        ] {
            let response = error.error_response();
            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            let bytes = response.into_body().try_into_bytes().unwrap_or_default();
            let text = String::from_utf8_lossy(&bytes);
            assert!(!text.contains("secret"), "leaked: {text}");
            assert!(text.contains("\"code\":\"internal_error\""), "{text}");
        }
    }
}
