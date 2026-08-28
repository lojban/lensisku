use super::models::{ImageData, SoundData};
use crate::export::models::CollectionExportItem;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListCollectionsQuery {
    /// Sort order for collections.
    /// - `active_week`  – most active flashcard users in the last 7 days (default)
    /// - `active_month` – most active flashcard users in the last 30 days
    /// - `active_all`   – most active flashcard users of all time
    /// - `newest`       – most recently created first
    pub sort: Option<String>,
    /// 1-based page index. If omitted with `per_page`, defaults to page 1.
    pub page: Option<i64>,
    /// Number of items per page (clamped server-side).
    pub per_page: Option<i64>,
    /// Case-insensitive substring search over collection name/description.
    pub search: Option<String>,
    /// When true, only return collections that already have at least one flashcard.
    pub has_flashcards_only: Option<bool>,
    /// When true, only return collections that have at least one flashcard level.
    pub has_levels_only: Option<bool>,
}

/// Body for `POST /collections/membership`: which of the caller's collections already contain
/// this definition (by `definition_id`) or this collection item (full content match).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CollectionMembershipRequest {
    /// Dictionary definition: membership is `collection_items.definition_id`.
    pub definition_id: Option<i32>,
    /// Collection item from search/results: membership is valsi + definition text +
    /// source language + target language (not `definition_id` alone).
    pub item_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionMembershipResponse {
    /// Caller's collections that already include a matching item, most recently updated first.
    pub collection_ids: Vec<i32>,
}

/// Body for `POST /collections/membership/batch`. Results are returned in the same order as `items`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CollectionMembershipBatchRequest {
    pub items: Vec<CollectionMembershipRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CollectionMembershipBatchResponse {
    /// Parallel to `items` in the request.
    pub results: Vec<CollectionMembershipResponse>,
}

/// One search-hit row's membership in the caller's collections (optional on dictionary search).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchHitMembership {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item_id: Option<i32>,
    pub collection_ids: Vec<i32>,
}

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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
    pub has_flashcards: bool,
    /// True when the collection has a cover image (`collection_images`).
    pub has_cover_image: bool,
    /// True when there is a cover or at least one item with a front/back card image (Tiktoknu / visual study).
    pub has_collection_image: bool,
    pub owner: CollectionOwner,
    /// Number of discussion comments on this collection (if any thread exists).
    #[serde(default)]
    pub comment_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CollectionOwner {
    pub user_id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
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
    /// When true, only items that have a front or back image
    pub has_card_image_only: Option<bool>,
    /// Comma-separated definition language ids (matches `definitions.langid`).
    pub languages: Option<String>,
    /// Filter by selmaho on the linked definition.
    pub selmaho: Option<String>,
    /// Filter by valsi typeid (word type) on the linked definition.
    pub word_type: Option<i16>,
    /// Comma-separated definition author usernames. When set, only those authors' definitions are included.
    pub username: Option<String>,
    /// Comma-separated usernames whose definitions should be excluded.
    pub exclude_usernames: Option<String>,
    /// Filter by valsi source language id (defaults to 1 = Lojban when unset).
    pub source_langid: Option<i32>,
    /// Mirrors dictionary search: when explicitly false, exclude phrase-typed valsi (typeid 15).
    pub search_in_phrases: Option<bool>,
    /// When true, rank definition-backed items by semantic similarity to `search`. Custom-text-only
    /// items (no `definition_id`) are appended after ranked rows in their natural order so they
    /// remain visible.
    pub semantic: Option<bool>,
}

/// Query for `GET /collections/users-and-collections`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CollectionUserPickerQuery {
    pub search: Option<String>,
    /// Max rows per kind when searching (collections and users). Clamped server-side (default 20, max 40).
    /// Empty search always returns the cached top 10 popular collections and top 10 recent authors.
    pub per_kind: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind")]
pub enum CollectionUserPickerItem {
    #[serde(rename = "collection")]
    Collection {
        collection_id: i32,
        name: String,
        owner_username: String,
    },
    #[serde(rename = "user")]
    User {
        user_id: i32,
        username: String,
        realname: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CollectionUserPickerResponse {
    pub items: Vec<CollectionUserPickerItem>,
}

/// Query for `GET /collections/items/search` — one request across a set of collections.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchCollectionsItemsQuery {
    /// Comma-separated collection ids. Private collections the caller cannot read are skipped.
    pub collection_ids: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub search: Option<String>,
    /// Comma-separated definition language ids (matches `definitions.langid`).
    pub languages: Option<String>,
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    pub username: Option<String>,
    pub exclude_usernames: Option<String>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    /// When true, rank definition-backed items by semantic similarity to `search`.
    pub semantic: Option<bool>,
}

/// Parse comma-separated positive ints, de-duplicated, order preserved. Caps at `max`.
pub fn parse_positive_id_list(value: &str, max: usize) -> Vec<i32> {
    let mut seen = std::collections::HashSet::new();
    let mut ids = Vec::new();
    for part in value.split(',') {
        let Ok(id) = part.trim().parse::<i32>() else {
            continue;
        };
        if id > 0 && seen.insert(id) {
            ids.push(id);
            if ids.len() >= max {
                break;
            }
        }
    }
    ids
}

/// Resolved filter inputs for `service::list_collection_items`. Built by the controller from
/// `ListCollectionItemsQuery` (parsing `languages`, materialising the semantic embedding, etc.)
/// so the service stays SQL-only.
#[derive(Debug, Default, Clone)]
pub struct ListCollectionItemsFilters {
    pub languages: Option<Vec<i32>>,
    pub selmaho: Option<String>,
    pub word_type: Option<i16>,
    pub usernames: Option<Vec<String>>,
    pub exclude_usernames: Option<Vec<String>>,
    pub source_langid: Option<i32>,
    pub search_in_phrases: Option<bool>,
    /// Pre-computed query embedding when semantic mode is active. The service treats it as the
    /// signal that semantic ranking should be applied; the controller is responsible for deciding
    /// whether to compute it.
    pub semantic_embedding: Option<pgvector::Vector>,
    /// When set, only return items whose valsi + definition text + source language + target
    /// language match this collection item (used to expand a deduped search hit).
    pub match_item_id: Option<i32>,
    /// Collapse identical valsi+definition+languages to one row *before* LIMIT (filtered search).
    pub dedupe_by_content: bool,
}

/// Lojban text → Ogg Opus via Kitten TTS (authenticated, rate-limited).
#[derive(Debug, Deserialize, ToSchema)]
pub struct KittenTtsGenerateRequest {
    /// Lojban text to convert to IPA and synthesize.
    pub text: String,
    /// Voice name (e.g. Bella, Bruno) or `expr-voice-*` embedding key.
    pub voice: String,
    /// Speech speed; clamped server-side (default 1.0).
    pub speed: Option<f32>,
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
    /// Set when the row was returned from a multi-collection search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<i32>,
    /// Display name of `collection_id` when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_langid: Option<i32>,
    #[schema(value_type = Option<String>, format = DateTime)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collection_created_at: Option<DateTime<Utc>>,
    /// How many collection items share this valsi+definition+languages (deduped search hits).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_count: Option<i32>,
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

/// One row for bulk edit: items with no dictionary definition and non-empty custom front and back text.
#[derive(Debug, Serialize, ToSchema)]
pub struct CustomTextBulkItemRow {
    pub item_id: i32,
    pub position: i32,
    pub free_content_front: String,
    pub free_content_back: String,
    pub language_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomTextBulkListResponse {
    pub items: Vec<CustomTextBulkItemRow>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CustomTextBulkUpdateItem {
    pub item_id: i32,
    pub free_content_front: String,
    pub free_content_back: String,
    pub language_id: Option<i32>,
}

/// New custom-text-only rows to append (no `item_id` yet); same semantics as `POST /collections/{id}/items` without definition.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CustomTextBulkNewItem {
    pub free_content_front: String,
    pub free_content_back: String,
    pub language_id: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CustomTextBulkUpdateRequest {
    #[serde(default)]
    pub items: Vec<CustomTextBulkUpdateItem>,
    #[serde(default)]
    pub new_items: Vec<CustomTextBulkNewItem>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CustomTextBulkUpdateResponse {
    pub updated: i32,
    pub inserted: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkRemoveItemsRequest {
    pub item_ids: Vec<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkRemoveItemsResponse {
    pub deleted: i32,
}

/// Adds many existing dictionary definitions to a collection in one transaction.
/// Used by the "Add all to collection" action on search-result pages.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkAddDefinitionsRequest {
    /// Definition IDs to copy into the collection.
    pub definition_ids: Vec<i32>,
    /// Optional note applied to every inserted row (HTML sanitized server-side).
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkAddDefinitionsResponse {
    /// Number of new `collection_items` rows inserted.
    pub added: i32,
    /// Definitions that were already in the collection (idempotent; not re-inserted).
    pub skipped: i32,
    /// Definition IDs rejected because they do not exist in the database.
    pub invalid_definition_ids: Vec<i32>,
}

/// One row in `manifest.json` for `POST /collections/{id}/items/media-bulk` (multipart) or `.../media-bulk-zip`.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MediaBulkManifestEntry {
    /// Basename only; must match an uploaded file (multipart filename or ZIP entry basename).
    pub filename: String,
    /// `front` or `back`.
    pub side: String,
    #[serde(default)]
    pub item_id: Option<i32>,
    /// Matches `collection_items.position` for this collection.
    #[serde(default)]
    pub position: Option<i32>,
    /// Required when both `item_id` and `position` are omitted (creates a new custom-text item).
    #[serde(default)]
    pub free_content_front: Option<String>,
    #[serde(default)]
    pub free_content_back: Option<String>,
    #[serde(default)]
    pub language_id: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MediaBulkImportResponse {
    pub attached: u32,
    pub created_items: u32,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::parse_positive_id_list;

    #[test]
    fn parse_positive_id_list_dedupes_skips_invalid_and_caps() {
        assert_eq!(
            parse_positive_id_list("1, 2, 2, -1, abc, 3", 50),
            vec![1, 2, 3]
        );
        assert_eq!(parse_positive_id_list("1,2,3,4", 2), vec![1, 2]);
        assert!(parse_positive_id_list("", 50).is_empty());
        assert!(parse_positive_id_list("0, -3", 50).is_empty());
    }
}
