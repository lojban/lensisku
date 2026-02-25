use super::models::{ImageData, SoundData};
use crate::export::models::CollectionExportItem;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCollectionRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MergeCollectionsRequest {
    pub source_collection_id: i32,
    pub target_collection_id: i32,
    pub new_collection_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionResponse {
    pub collection_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<Utc>,
    pub item_count: i64,
    pub owner: CollectionOwner,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionOwner {
    pub user_id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionListResponse {
    pub collections: Vec<CollectionResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionItemListResponse {
    pub items: Vec<CollectionItemResponse>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCollectionItemsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    #[schema(example = 123)]
    pub item_id: Option<i32>,
    /// Filter items that have no associated flashcards
    pub exclude_with_flashcards: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemPositionRequest {
    pub position: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemNotesRequest {
    pub notes: Option<String>,
    pub auto_progress: Option<bool>,
}

// TODO (Applicative Refactor): Validation logic using this payload is a candidate
// for applicative-style error handling to collect multiple validation errors
// into AppError::Validation instead of short-circuiting. Potential validations:
// - item_id XOR definition_id XOR free_content fields should be set
// - position should be non-negative if present
// - direction should be a valid value if present
// - image data size/format validation
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddItemRequest {
    pub item_id: Option<i32>,
    pub definition_id: Option<i32>,
    pub notes: Option<String>,
    pub position: Option<i32>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub direction: Option<String>,
    pub language_id: Option<i32>,
    pub owner_user_id: Option<i32>,
    pub license: Option<String>,
    pub script: Option<String>,
    pub is_original: Option<bool>,
    #[serde(default, rename = "auto_progress")]
    pub auto_progress: Option<bool>,
    #[schema(format = "binary")]
    pub front_image: Option<ImageData>,
    #[schema(format = "binary")]
    pub back_image: Option<ImageData>,
    #[schema(format = "binary")]
    pub sound: Option<SoundData>,
    /// When updating an item, set to true to remove existing custom sound
    pub remove_sound: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct FlashcardResponse {
    pub id: i32,
    pub direction: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<Utc>,
    pub canonical_form: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionItemResponse {
    pub lang_id: Option<i32>,
    pub item_id: i32,
    pub definition_id: Option<i32>,
    pub word: Option<String>,
    pub username: Option<String>,
    pub valsi_id: Option<i32>,
    pub definition: Option<String>,
    pub free_content_front: Option<String>,
    pub free_content_back: Option<String>,
    pub notes: Option<String>,
    pub language_id: Option<i32>,
    pub owner_user_id: Option<i32>,
    pub license: Option<String>,
    pub script: Option<String>,
    pub is_original: bool,
    pub ci_notes: Option<String>,
    pub position: i32,
    pub auto_progress: bool,
    pub has_front_image: bool,
    pub has_back_image: bool,
    pub has_sound: bool,
    /// Custom sound or DB valsi sound URL only (no external URLs): /api/collections/{id}/items/{item_id}/sound or /api/jbovlaste/valsi/{word}/sound.
    pub sound_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub added_at: DateTime<Utc>,
    pub canonical_form: Option<String>,
    pub flashcard: Option<FlashcardResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemRequest {
    pub notes: Option<String>,
    #[schema(format = "binary")]
    pub front_image: Option<ImageData>,
    #[schema(format = "binary")]
    pub back_image: Option<ImageData>,
    pub remove_front_image: Option<bool>,
    pub remove_back_image: Option<bool>,
    #[schema(format = "binary")]
    pub sound: Option<SoundData>,
    pub remove_sound: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportJsonRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
    pub items: Vec<ImportJsonItem>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportJsonItem {
    pub word: String,
    pub definition_id: Option<i32>,
    pub collection_note: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportJsonResponse {
    pub collection: CollectionResponse,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SearchItemsResponse {
    pub items: Vec<CollectionItemResponse>,
    pub total: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportCollectionJsonRequest {
    pub items: Vec<CollectionExportItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportCollectionJsonResponse {
    pub imported_count: i32,
    pub skipped_count: i32,
    pub skipped_items: Vec<SkippedItemInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SkippedItemInfo {
    pub identifier: String, // e.g., definition_id or free_content_front
    pub reason: String,
}

// Full collection export/import (items + levels + optional flashcards)

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionExportMeta {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionFullExport {
    pub collection: CollectionExportMeta,
    pub items: Vec<CollectionExportItem>,
    pub levels: Vec<LevelExport>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct LevelExport {
    pub name: String,
    pub description: Option<String>,
    pub min_cards: i32,
    pub min_success_rate: f32,
    pub position: i32,
    /// Indices into the levels array (0-based) for prerequisites
    #[serde(default)]
    pub prerequisite_positions: Vec<usize>,
    /// Indices into the items array for which items' flashcards belong to this level
    #[serde(default)]
    pub item_positions: Vec<usize>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportFullRequest {
    pub collection: ImportFullCollectionMeta,
    pub items: Vec<CollectionExportItem>,
    #[serde(default)]
    pub levels: Vec<LevelExport>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportFullCollectionMeta {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ImportFullResponse {
    pub collection: CollectionResponse,
    pub imported_count: i32,
    pub skipped_count: i32,
    pub levels_created: i32,
    pub warnings: Vec<String>,
}
