use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::models::Comment;

#[derive(Debug)]
pub struct NewCommentParams {
    pub pool: Pool,
    pub user_id: i32,
    pub valsi_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub target_user_id: Option<i32>,
    pub parent_id: Option<i32>,
    pub subject: String,
    pub content: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CommentActionRequest {
    pub comment_id: i32,
    pub action: bool, // true to like/bookmark, false to unlike/unbookmark
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentResponse {
    pub comment: super::models::Comment,
    pub replies: i64,
    pub is_bookmarked: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ThreadResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOpinionRequest {
    pub comment_id: i32,
    pub opinion: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpinionVoteRequest {
    pub opinion_id: i64,
    pub comment_id: i32,
    pub vote: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentStats {
    pub total_bookmarks: i64,
    pub total_replies: i64,
    pub total_opinions: i64,
    pub total_reactions: i64,
    #[schema(value_type = String, format = DateTime)]
    pub last_activity_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ContentPart {
    #[schema(example = "text")]
    pub r#type: String,
    #[schema(example = "Hello world!")]
    pub data: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NewCommentRequest {
    #[serde(default)]
    pub valsi_id: Option<i32>,
    #[serde(default)]
    pub natlang_word_id: Option<i32>,
    #[serde(default)]
    pub target_user_id: Option<i32>,
    #[serde(default)]
    pub definition_id: Option<i32>,
    #[serde(default)]
    pub definition_link_id: Option<i32>,
    #[serde(default)]
    pub parent_id: Option<i32>, // None or 0 for top-level comments
    pub subject: String,
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrendingHashtag {
    pub tag: String,
    pub usage_count: i64,
    #[schema(value_type = String, format = DateTime)]
    pub last_used: DateTime<Utc>,
}

impl From<tokio_postgres::Row> for TrendingHashtag {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            tag: row.get("tag"),
            usage_count: row.get("usage_count"),
            last_used: row.get("last_used"),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReactionRequest {
    pub comment_id: i32,
    pub reaction: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ReactionResponse {
    pub reaction: String,
    pub count: i64,
    pub reacted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedReactions {
    pub reactions: Vec<ReactionResponse>,
    pub total_reactions: i64,
    pub total_pages: i64,
    pub current_page: i64,
    pub page_size: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReactionSummary {
    pub reactions: PaginatedReactions,
    pub total_distinct_reactions: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReactionPaginationQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 10)]
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedCommentsResponse {
    pub comments: Vec<Comment>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedUserCommentsResponse {
    pub comments: Vec<Comment>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct FreeThreadQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    #[schema(default = "time", example = "subject")]
    pub sort_by: Option<String>,
    #[schema(default = "desc", example = "asc")]
    pub sort_order: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ThreadQuery {
    pub valsi_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub target_user_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub scroll_to: Option<i32>,
    pub thread_id: Option<i32>,
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TrendingQuery {
    #[schema(default = "LastWeek", example = "week")]
    pub timespan: Option<String>,
    #[schema(default = 10)]
    pub limit: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PaginationQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchCommentsQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    pub search: Option<String>,
    #[schema(default = "time")]
    pub sort_by: Option<String>,
    #[schema(default = "desc")]
    pub sort_order: Option<String>,
    pub username: Option<String>,
    pub valsi_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub target_user_id: Option<i32>,
}

#[derive(Debug)]
pub struct SearchCommentsParams {
    pub page: i64,
    pub per_page: i64,
    pub search_term: String,
    pub sort_by: String,
    pub sort_order: String,
    pub username: Option<String>,
    pub valsi_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub target_user_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCommentsQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    #[schema(default = "desc", example = "asc")]
    pub sort_order: Option<String>,
}

#[derive(Debug)]
pub struct ThreadParams {
    pub thread_id: Option<i32>,
    pub valsi_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub target_user_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub scroll_to: Option<i32>,
    pub current_user_id: Option<i32>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}
