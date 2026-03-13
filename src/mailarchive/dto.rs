use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::Message;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub query: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub include_content: Option<bool>,
    pub group_by_thread: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchResponse {
    pub messages: Vec<Message>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ThreadQuery {
    pub subject: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub include_content: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ThreadResponse {
    pub messages: Vec<Message>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub clean_subject: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SpamVoteResponse {
    pub message_id: i32,
    pub spam_vote_count: i64,
    pub success: bool,
    pub user_voted: bool,
}

/// Summary of a mail thread (all messages with same cleaned_subject) for listing.
#[derive(Debug)]
pub struct MailThreadSummary {
    pub cleaned_subject: String,
    pub subject: Option<String>,
    pub from_address: Option<String>,
    pub last_sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub message_count: i64,
    pub content_preview: Option<String>,
}
