//! Typed errors for flashcard services (per-domain error enums instead of
//! matching on `e.to_string()` in controllers).
//!
//! `Display` reproduces the exact strings the old `Box<dyn Error>` values produced, so
//! response bodies stay byte-for-byte identical while the status mapping becomes a
//! `match` on variants.

use std::fmt;

#[derive(Debug)]
pub enum FlashcardError {
    /// The level does not exist (row count 0 on update/delete).
    LevelNotFound,
    /// Another resource is missing (e.g. a card in a level). Maps to 404.
    NotFound(String),
    /// Access to the collection was refused (read or ownership). Maps to 403.
    AccessDenied(String),
    /// A request the service refuses on purpose (e.g. deleting a prerequisite level).
    BadRequest(String),
    /// Anything else, rendered with its original message.
    Other(String),
    /// A wrapped lower-level error. `Display` and `source()` delegate to it, so
    /// callers that walk the source chain (e.g. to show the Postgres message
    /// behind "db error") see exactly what the old boxed error exposed.
    Source(Box<dyn std::error::Error>),
}

impl fmt::Display for FlashcardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlashcardError::LevelNotFound => f.write_str("Level not found"),
            FlashcardError::AccessDenied(msg)
            | FlashcardError::NotFound(msg)
            | FlashcardError::BadRequest(msg)
            | FlashcardError::Other(msg) => f.write_str(msg),
            FlashcardError::Source(e) => fmt::Display::fmt(e, f),
        }
    }
}

impl std::error::Error for FlashcardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FlashcardError::Source(e) => e.source(),
            _ => None,
        }
    }
}

impl From<tokio_postgres::Error> for FlashcardError {
    fn from(e: tokio_postgres::Error) -> Self {
        FlashcardError::Source(Box::new(e))
    }
}

impl From<deadpool_postgres::PoolError> for FlashcardError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        FlashcardError::Source(Box::new(e))
    }
}

impl From<crate::AppError> for FlashcardError {
    fn from(e: crate::AppError) -> Self {
        match e {
            crate::AppError::Auth(_) => FlashcardError::AccessDenied(e.to_string()),
            crate::AppError::NotFound(_) => FlashcardError::NotFound(e.to_string()),
            other => FlashcardError::Source(Box::new(other)),
        }
    }
}

/// Legacy string errors (`Err("...".into())`) keep their text and map to 500.
impl From<String> for FlashcardError {
    fn from(e: String) -> Self {
        FlashcardError::Other(e)
    }
}

impl From<&str> for FlashcardError {
    fn from(e: &str) -> Self {
        FlashcardError::Other(e.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for FlashcardError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        FlashcardError::Source(e)
    }
}

impl From<fsrs::FSRSError> for FlashcardError {
    fn from(e: fsrs::FSRSError) -> Self {
        FlashcardError::Source(Box::new(e))
    }
}

impl From<serde_json::Error> for FlashcardError {
    fn from(e: serde_json::Error) -> Self {
        FlashcardError::Source(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::FlashcardError;

    #[test]
    fn display_matches_legacy_strings() {
        assert_eq!(FlashcardError::LevelNotFound.to_string(), "Level not found");
        let auth = FlashcardError::from(crate::AppError::Auth("Access denied".into()));
        assert!(matches!(auth, FlashcardError::AccessDenied(_)));
        assert_eq!(
            auth.to_string(),
            "Authentication/Authorization error: Access denied"
        );
    }
}
