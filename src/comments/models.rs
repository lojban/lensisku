use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;

use super::dto::ReactionResponse;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommentContent {
    #[schema(value_type = String)]
    pub r#type: String,
    pub data: String,
}

impl postgres_types::ToSql for CommentContent {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut bytes::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let value = serde_json::to_value(self)?;
        postgres_types::Json(value).to_sql(ty, out)
    }

    postgres_types::accepts!(JSONB);
    postgres_types::to_sql_checked!();
}

impl postgres_types::FromSql<'_> for CommentContent {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value = postgres_types::FromSql::from_sql(ty, raw)?;
        Ok(serde_json::from_value(value)?)
    }

    postgres_types::accepts!(JSONB);
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Comment {
    pub valsi_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub comment_id: i32,
    pub thread_id: i32,
    pub parent_id: Option<i32>,
    pub user_id: i32,
    pub comment_num: i32,
    pub time: i32,
    pub subject: String,
    #[schema(value_type = Vec<CommentContent>)]
    pub content: Vec<CommentContent>,
    pub username: Option<String>,
    pub last_comment_username: Option<String>,
    pub realname: Option<String>,
    pub total_reactions: i64,
    pub total_replies: i64,
    pub is_bookmarked: Option<bool>,
    #[schema(value_type = Vec<ReactionResponse>)]
    pub reactions: Vec<ReactionResponse>,
    #[schema(value_type = Option<Vec<CommentContent>>)]
    pub parent_content: Option<Vec<CommentContent>>,
    pub valsi_word: Option<String>,
    pub definition: Option<String>,
    pub first_comment_subject: Option<String>,
    #[schema(value_type = Option<Vec<CommentContent>>)]
    pub first_comment_content: Option<Vec<CommentContent>>,
}

impl Comment {
    pub fn extract_hashtags(content: &str) -> Result<HashSet<String>, regex::Error> {
        let re = Regex::new(r"#(\w+)")?;
        Ok(re
            .captures_iter(content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_lowercase())
            .collect())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Thread {
    pub thread_id: i32,
    pub valsi_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub target_user_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub valsi: Option<String>,
    pub natlang_word: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentOpinion {
    pub id: i64,
    pub opinion: String,
    pub comment_id: i32,
    pub user_id: i32,
    pub votes: i32,
    pub voted: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

impl CommentOpinion {
    const OPINION_MAX_LEN: usize = 12;

    pub fn parse(content: &str) -> Option<String> {
        let new_string = content.to_lowercase();
        let count = UnicodeSegmentation::graphemes(new_string.as_str(), true).count();

        if content.is_empty() || count > Self::OPINION_MAX_LEN {
            return None;
        }

        Some(new_string)
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CommentWithOpinions {
    #[schema(inline)]
    pub comment: Comment,
    pub opinions: Vec<CommentOpinion>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CommentReaction {
    pub id: i32,
    pub comment_id: i32,
    pub user_id: i32,
    pub reaction: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum TrendingTimespan {
    LastDay,
    LastWeek,
    LastMonth,
    LastYear,
    AllTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FreeThread {
    pub thread_id: i32,
    pub valsiid: Option<i32>,
    pub definitionid: Option<i32>,
    pub target_user_id: Option<i32>,
    pub definition_link_id: Option<i32>,
    pub valsi_word: Option<String>,
    pub definition: Option<String>,
    pub last_comment_id: i32,
    pub last_comment_time: i32,
    pub last_comment_subject: String,
    #[schema(value_type = Vec<CommentContent>)]
    pub last_comment_content: Vec<CommentContent>,
    pub first_comment_subject: String,
    #[schema(value_type = Vec<CommentContent>)]
    pub first_comment_content: Vec<CommentContent>,
    pub total_comments: i64,
    pub last_comment_username: Option<String>,
    pub username: String,
    pub realname: Option<String>,
    pub is_bookmarked: Option<bool>,
    pub user_id: i32,
    pub comment_num: i32,
    pub parent_id: Option<i32>,
    pub total_reactions: i64,
    #[schema(value_type = Vec<ReactionResponse>)]
    pub reactions: Vec<ReactionResponse>,
}
