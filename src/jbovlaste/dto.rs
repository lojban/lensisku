use super::{models::KeywordMapping, DefinitionDetail, RecentChange};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

#[derive(Deserialize, ToSchema)]
pub struct SearchDefinitionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub include_comments: Option<bool>,
    pub languages: Option<String>,
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    pub username: Option<String>,
    pub source_langid: Option<i32>,
    pub fast: Option<bool>,
    pub search_in_phrases: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct NonLojbanDefinitionsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub languages: Option<String>,  // Filter by definition language
    pub username: Option<String>,   // Filter by definition author username
    pub source_langid: Option<i32>, // Filter by the source language of the valsi
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListDefinitionsQuery {
    #[schema(default = 1)]
    pub page: Option<i64>,
    #[schema(default = 20)]
    pub per_page: Option<i64>,
    pub search: Option<String>,
    #[schema(default = "created_at", example = "updated_at")]
    pub sort_by: Option<String>,
    #[schema(default = "desc", example = "asc")]
    pub sort_order: Option<String>,
    pub languages: Option<String>, // Comma-separated list of langids
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    pub user_id: Option<i32>,
    pub source_langid: Option<i32>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct DefinitionListResponse {
    pub definitions: Vec<DefinitionDetail>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decomposition: Vec<String>,
}
#[derive(Debug, Deserialize, ToSchema)]
pub struct ValsiDefinitionsQuery {
    pub langid: Option<i32>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GetImageDefinitionQuery {
    pub image_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddDefinitionRequest {
    pub word: String,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub lang_id: i32,
    pub source_langid: Option<i32>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub rafsi: Option<String>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub owner_only: Option<bool>,
    #[schema(format = "binary")]
    pub image: Option<ImageData>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AddValsiResponse {
    pub success: bool,
    pub word_type: String,
    pub definition_id: i32,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDefinitionRequest {
    pub lang_id: i32,
    pub definition: String,
    pub notes: Option<String>,
    pub etymology: Option<String>,
    pub gloss_keywords: Option<Vec<KeywordMapping>>,
    pub place_keywords: Option<Vec<KeywordMapping>>,
    pub selmaho: Option<String>,
    pub jargon: Option<String>,
    pub rafsi: Option<String>,
    pub owner_only: Option<bool>,
    #[schema(format = "binary")]
    pub image: Option<ImageData>,
    pub remove_image: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateDefinitionResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VoteRequest {
    pub definition_id: i32,
    pub downvote: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoteResponse {
    pub success: bool,
    pub message: String,
    pub word: Option<String>,
    pub score: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserVoteResponse {
    pub vote: Option<i32>, // 1 for upvote, -1 for downvote, None if no vote
    pub definition_id: i32,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct BulkVoteRequest {
    #[validate(
        length(
            max = 1000,
            message = "Cannot request more than 1000 definitions at once"
        ),
        custom(function = "validate_unique_sorted")
    )]
    pub definition_ids: Vec<i32>,
}

fn validate_unique_sorted(ids: &[i32]) -> Result<(), ValidationError> {
    let mut prev = None;
    for id in ids {
        if prev >= Some(id) {
            return Err(ValidationError::new(
                "Definition IDs must be unique and sorted",
            ));
        }
        prev = Some(id);
    }
    Ok(())
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkVoteResponse {
    pub votes: std::collections::HashMap<String, Option<i32>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClientIdGroup {
    pub client_id: String,
    pub count: i64, // Using i64 for count, consistent with pagination totals
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VoteError {
    pub error: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RecentChangesResponse {
    pub changes: Vec<RecentChange>,
    pub total: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecentChangesQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkImportRequest {
    /// CSV content with columns: gismu,definition,notes,glosswords
    #[schema(format = "binary")]
    pub csv: String,
    /// Target language ID for all definitions
    pub lang_id: i32,
}

#[derive(Debug)]
pub struct BulkImportParams<'a> {
    pub csv_data: &'a str,
    pub lang_id: i32,
    pub client_id: String,
    pub import_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageUploadRequest {
    #[schema(format = "binary")]
    pub image: ImageData,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageData {
    #[schema(format = "byte")]
    pub data: String, // Base64 encoded image data
    pub mime_type: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LinkDefinitionsRequest {
    pub definition_id: i32,
    pub translation_id: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DefinitionTranslation {
    pub definitionid: i32,
    pub valsiword: String,
    pub definition: String,
    pub langid: i32,
    pub lang_name: String,
    pub link_id: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExportPairsQuery {
    pub from_lang: i32,
    pub to_lang: i32,
}
