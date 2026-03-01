use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use utoipa::ToSchema;

use crate::comments::dto::ReactionResponse;
use crate::versions::VersionDiff;

#[derive(Debug)]
pub struct SearchDefinitionsParams {
    pub page: i64,
    pub per_page: i64,
    pub search_term: String,
    pub include_comments: bool,
    pub sort_by: String,
    pub sort_order: String,
    pub languages: Option<Vec<i32>>,
    pub selmaho: Option<String>,
    pub username: Option<String>,
    pub word_type: Option<i16>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiEntry {
    pub valsiid: i32,
    pub word: String,
    pub type_id: i16,
    pub type_name: String,
    pub rafsi: Option<String>,
    pub langid: Option<i32>,
    pub comment_count: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiDetail {
    pub valsiid: i32,
    pub word: String,
    pub type_name: String,
    pub rafsi: Option<String>,
    pub comment_count: Option<i64>,
    pub source_langid: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable)]
    pub decomposition: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionDetail {
    pub definitionid: i32,
    pub valsiword: String,
    pub valsiid: i32,
    pub langid: i32,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub definitionnum: i32,
    pub langrealname: String,
    pub username: String,
    pub time: i32,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub type_name: String,
    pub rafsi: Option<String>,
    pub score: f32,
    pub comment_count: Option<i64>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub user_vote: Option<i32>,
    pub owner_only: bool,
    pub can_edit: bool,
    pub has_image: bool,
    pub sound_url: Option<String>,
    #[schema(value_type = Vec<f32>)]
    pub embedding: Option<Vec<f32>>,
    pub similarity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Example>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Example {
    pub exampleid: i32,
    pub content: String,
    pub examplenum: i32,
    pub username: String,
    pub time: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionResponse {
    pub definitions: Vec<DefinitionDetail>,
    pub decomposition: Vec<String>,
    pub total: i64,
}

impl From<tokio_postgres::Row> for DefinitionDetail {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            similarity: row
                .get::<_, Option<f64>>("similarity")
                .map(|d| (1.0 - d).max(0.0)),
            definitionid: row.get("definitionid"),
            valsiword: row.get("valsiword"),
            valsiid: row.get("valsiid"),
            langid: row.get("langid"),
            definition: row.get("definition"),
            notes: row.get("notes"),
            etymology: row.get("etymology"),
            selmaho: row.get("selmaho"),
            jargon: row.get("jargon"),
            definitionnum: row.get("definitionnum"), // Note the field name matches DB
            langrealname: row.get("langrealname"),
            username: row.get("username"),
            time: row.get("time"),
            type_name: row.get("type_name"),
            rafsi: row.try_get("rafsi").ok().flatten(), // Use try_get in case column is missing in some queries
            score: row.get("score"),
            user_vote: row.get("user_vote"),
            comment_count: row.get("comment_count"),
            gloss_keywords: None, // These get filled in separately
            place_keywords: None,
            owner_only: row.get("owner_only"),
            can_edit: row.get("can_edit"),
            created_at: row.get("created_at"),
            has_image: row.get("has_image"),
            sound_url: row.get("sound_url"),
            embedding: None,
            metadata: row.get("metadata"),
            examples: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct KeywordMapping {
    pub word: String,
    pub meaning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecentChange {
    pub change_type: String, // valsi, definition, comment
    pub word: String,
    pub content: serde_json::Value,
    pub valsi_id: Option<i32>,
    pub lang_id: Option<i32>,
    pub natlang_word_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub thread_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub username: String,
    pub time: i32,
    pub language_name: Option<String>,
    pub diff: Option<VersionDiff>,
    // Comment-type only (optional so non-comment changes stay small)
    pub comment_num: Option<i32>,
    pub valsi_word: Option<String>,
    pub parent_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reactions: Option<Vec<ReactionResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bookmarked: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiType {
    pub type_id: i16,
    pub descriptor: String,
}

impl From<tokio_postgres::Row> for ValsiType {
    fn from(row: tokio_postgres::Row) -> Self {
        ValsiType {
            type_id: row.get("typeid"),
            descriptor: row.get("descriptor"),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ValsiTypeListResponse {
    pub types: Vec<ValsiType>,
}
