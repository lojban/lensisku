use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Single wiki search hit (rendered into `WaveSearchHit::Wiki`).
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WikiSearchHit {
    pub page_id: i32,
    pub namespace: i32,
    pub title: String,
    pub last_edited: Option<DateTime<Utc>>,
    /// Short plain-text snippet (~400 chars) for list display.
    pub content_preview: Option<String>,
    /// URL to render the article (internal SPA route).
    pub article_url: String,
}

/// Summary in the unified threads list (one wiki article = one "thread").
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WikiThreadSummary {
    pub page_id: i32,
    pub namespace: i32,
    pub title: String,
    pub last_edited: Option<DateTime<Utc>>,
    pub content_preview: Option<String>,
    pub article_url: String,
}

/// Full article payload returned by `GET /wiki/{title}`.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WikiArticleDetail {
    pub page_id: i32,
    pub namespace: i32,
    pub title: String,
    pub markdown: String,
    pub last_edited: Option<DateTime<Utc>>,
    pub is_redirect: bool,
    /// Direct link back to mw.lojban.org for "view source".
    pub source_url: String,
}
