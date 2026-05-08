use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::comments::models::Comment;
use crate::mailarchive::Message;
use crate::wiki::dto::{WikiSearchHit, WikiThreadSummary};

/// Single search hit: either a comment or a mail message.
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum WaveSearchHit {
    Comment {
        comment: Comment,
        #[serde(skip_serializing_if = "Option::is_none")]
        import_source: Option<String>,
    },
    Mail {
        message: Message,
    },
    Wiki {
        article: WikiSearchHit,
    },
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WavesSearchQuery {
    pub search: Option<String>,
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    #[schema(default = "time")]
    pub sort_by: Option<String>,
    #[schema(default = "desc")]
    pub sort_order: Option<String>,
    /// `all` | `jbotcan` | `comments` | `mail` | `wiki`
    pub source: Option<String>,
    /// Restrict comment hits to threads attached to this collection. When set, the mail and wiki
    /// halves of the unified search are suppressed because they are not collection-scoped.
    pub collection_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WavesSearchResponse {
    pub items: Vec<WaveSearchHit>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Summary of a single thread (comment thread or mail thread) for the threads list.
#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum WaveThreadSummary {
    Comment {
        thread_id: i32,
        comment_id: i32,
        #[serde(skip_serializing_if = "Option::is_none")]
        import_source: Option<String>,
        first_comment_subject: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        first_comment_content: Option<serde_json::Value>,
        username: Option<String>,
        last_comment_username: Option<String>,
        last_activity_time: i32,
        total_replies: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        simple_content: Option<String>,
        valsi_id: Option<i32>,
        definition_id: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        valsi_word: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        definition: Option<String>,
        /// Reactions on the last comment in the thread (for sorting / display).
        last_comment_reactions: i64,
        last_comment_parent_id: Option<i32>,
        comment_num: i32,
    },
    Mail {
        cleaned_subject: String,
        subject: Option<String>,
        from_address: Option<String>,
        last_activity_time: i64,
        message_count: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        content_preview: Option<String>,
        /// For unified sort with comment threads (mail has no reactions).
        last_comment_reactions: i64,
    },
    Wiki {
        summary: WikiThreadSummary,
        /// Unix timestamp of `last_edited` (or 0) for unified sort.
        last_activity_time: i64,
        /// Always 0 (wiki has no reactions); kept for unified sort.
        last_comment_reactions: i64,
    },
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WavesThreadsQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    #[schema(default = "time")]
    pub sort_by: Option<String>,
    #[schema(default = "desc")]
    pub sort_order: Option<String>,
    /// `all` | `jbotcan` | `comments` | `mail` | `wiki`
    pub source: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WavesThreadsResponse {
    pub items: Vec<WaveThreadSummary>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}
