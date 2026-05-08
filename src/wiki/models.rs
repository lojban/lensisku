use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use utoipa::ToSchema;

/// Row from `wiki_articles` (raw columns; UI-facing DTOs live in [`super::dto`]).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WikiArticle {
    pub id: i32,
    pub page_id: i32,
    pub namespace: i32,
    pub title: String,
    pub revision_id: Option<i64>,
    pub markdown: String,
    pub plain_text: String,
    pub is_redirect: bool,
    pub last_edited: Option<DateTime<Utc>>,
}

impl From<Row> for WikiArticle {
    fn from(row: Row) -> Self {
        WikiArticle {
            id: row.get("id"),
            page_id: row.get("page_id"),
            namespace: row.get("namespace"),
            title: row.get("title"),
            revision_id: row.try_get("revision_id").unwrap_or(None),
            markdown: row.get("markdown"),
            plain_text: row.get("plain_text"),
            is_redirect: row.try_get("is_redirect").unwrap_or(false),
            last_edited: row.try_get("last_edited").unwrap_or(None),
        }
    }
}
